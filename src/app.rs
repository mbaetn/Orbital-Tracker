use chrono::{DateTime, Utc};
use std::{env, time::Duration};
use std::error::Error;
use dotenv::dotenv;
use crossterm::{
    event::{poll, read, Event, KeyEvent}
};


mod api;
use api::ApiClient;
use api::TleData;
mod physic;
use physic::SGP4Propagator;
use crate::app::interface::coordinates::LongLatCoordinates;
mod interface;
use interface::Interface;
mod satellite_entity;
use satellite_entity::SatelliteEntity;
mod commands;
use commands::Command;

pub struct App{
    selected_index: usize,
    page_index: usize,
    satellites: Vec<SatelliteEntity>,
    history: Vec<LongLatCoordinates>,
    should_quit: bool,
    visible_rows: usize,
    is_searching: bool,
    search_query: String,
    filtered_indices: Vec<usize>,
}

pub struct SatelliteState{
    name:String,
    pos: LongLatCoordinates
}

#[derive(Clone, Copy)]
pub struct SearchState<'a> {
    pub is_searching: bool,
    pub query: &'a str,
}


impl App{

    pub fn init()-> Result<Self, Box<dyn Error>>{
        dotenv().ok();
        let api_key = env::var("API_KEY").expect("No API_KEY found");
        let base_url = "https://api.keeptrack.space/v4/sats";

        let api =  ApiClient::new(base_url, &api_key);
        let tle_satellites:Vec<TleData> = api.get_satellites_tles()?;

        let satellites: Vec<SatelliteEntity> =  tle_satellites.into_iter().filter_map(|target| Self::to_satellite_entity(target).ok().flatten()).collect();

        let history = Vec::new();
        let should_quit = false;
        let selected_index:usize = 0;
        let visible_rows:usize = 15;
        let page_index:usize = 0;
        let is_searching = false;
        let search_query = String::new();
        let filtered_indices:Vec<usize> = (0..satellites.len()).collect();
        let res = Self{selected_index, page_index, satellites, history, should_quit, visible_rows, is_searching, search_query, filtered_indices};

        Ok(res)
    }

    fn to_satellite_entity(target: TleData) -> Result<Option<SatelliteEntity>, Box<dyn Error>>{
        let propagator = SGP4Propagator::new(&target)?;
        Ok(Some(SatelliteEntity::new(target, propagator)))
    }


    pub fn run(&mut self) -> Result<(), Box<dyn Error>>{

        let mut ui = Interface::new()?;


        self.update_history(45)?;
        while !self.should_quit{
            self.visible_rows = ui.get_visible_rows().max(1);

            let start_index = (self.page_index)*self.visible_rows;
            let tracked_positions: Vec<SatelliteState> = self.compute_positions(Utc::now(), start_index, start_index + self.visible_rows);
            let max_index = tracked_positions.len().saturating_sub(1);
            if tracked_positions.is_empty() {
                self.selected_index = 0;
            } else if self.selected_index > max_index {// Case when the window size is reduced
                  self.select_satellite(max_index)?;
            }




            // Update UI
            ui.set_selected_row(self.selected_index);
            let nb_pages = self.calculate_page_count();
            let search_state = SearchState{is_searching:self.is_searching, query: &self.search_query};
            ui.draw(tracked_positions, &self.history, self.page_index, nb_pages, search_state)?;


            if poll(Duration::from_millis(1000))? { // If nothing happens during dt, skip that part and refresh position
                if let Event::Key(key_event) = read()? {
                    self.handle_command(key_event);
                }
            }


        }
        ui.terminate()?;
        Ok(())

    }

    /// Satellite index in `filtered_satellites`
    fn active_satellite_index(&self) -> usize {
        let filtered_satellites:Vec<&SatelliteEntity> = self.filtered_indices.iter().map(|&i| self.satellites.get(i).unwrap()).collect();
        if self.filtered_indices.is_empty() {
            return 0;
        }
        let page_size = self.visible_rows.max(1);
        let raw = self.page_index.saturating_mul(page_size).saturating_add(self.selected_index);
        raw.min(filtered_satellites.len()-1)
    }

    fn calculate_page_count(&self)->usize{
        self.filtered_indices.len().div_ceil(self.visible_rows.max(1))
    }

    fn select_satellite(&mut self, index:usize) -> Result<(), Box <dyn Error>>{
        self.selected_index = index;
        self.update_history(45)?;
        Ok(())
    }

    /// Compute positions of satellites between `start_index` and `end_index`
    fn compute_positions(&self, now: DateTime<Utc>, start_index:usize, end_index:usize) -> Vec<SatelliteState>{
        let filtered_satellites:Vec<&SatelliteEntity> = self.filtered_indices.iter().map(|&i| self.satellites.get(i).unwrap()).collect();
        let end = end_index.min(filtered_satellites.len());
        filtered_satellites[start_index..end]
            .iter()
            .filter_map(|target| Self::extract_satellite_info(target, now))
            .collect()

    }



    /// Extract name and position in longitude/latitude from a `SatelliteEntity` using Option to manage errors due to SGP4
    /// (e.g outofrange excentricity) and return None
    fn extract_satellite_info(satellite: &SatelliteEntity, now: DateTime<Utc>) -> Option<SatelliteState>{
        let name = satellite.get_tle().name.as_ref()?;
        let propagator = satellite.get_propagator();
        let evaluate_motion = propagator.get_position(now).ok()?;
        let pos = LongLatCoordinates::from_cartesian(evaluate_motion.position, now);
        Some(SatelliteState{name: name.to_string(), pos})
    }

    fn update_satellites(&mut self) {
        let query = self.search_query.trim().to_lowercase();

        if query.is_empty() {
            self.filtered_indices = (0..self.satellites.len()).collect();
        } else {
            self.filtered_indices = self
                .satellites
                .iter()
                .enumerate()
                .filter_map(|(index, sat)| {
                    let name = sat.get_tle().name.as_deref()?;
                    name.to_lowercase().contains(&query).then_some(index)
                })
                .collect();
        }

        self.page_index = 0;
        self.selected_index = 0;
    }


    fn update_history(&mut self, timelaps:i64) -> Result<(), Box <dyn Error>>{
        let mut history = Vec::new();

        if self.filtered_indices.is_empty(){
            self.history.clear();
            return Ok(())
        }
        let active_index = self.active_satellite_index(); // active_satellite_index gives in filtered_satellites,
        let real_index = self.filtered_indices.get(active_index).copied().unwrap_or(0); // get the matching index in satellites
        let target = &self.satellites[real_index];



        let propagator = target.get_propagator();
        let now = Utc::now();
        for i in -timelaps..timelaps{
            let current_step_time = now + chrono::Duration::try_minutes(i).unwrap();
            if let Ok(prediction) = propagator.get_position(current_step_time) {
                let coords = LongLatCoordinates::from_cartesian(prediction.position, current_step_time);
                history.push(coords);
            }
        }

        self.history = history;
        Ok(())
    }


    // region command management
    fn handle_command(&mut self, event: KeyEvent){
        if let Ok(command) = Command::try_from(event){
            self.process_command(command);
        }
    }

    fn process_command(&mut self, command: Command){
        match command{
            Command::Quit=> self.should_quit = true,
            Command::Next=> self.process_next_command(),
            Command::Previous=> self.process_previous_command(),
            Command::NextPage => self.process_next_page_command(),
            Command::PreviousPage => self.process_previous_page_command(),
            Command::ToggleSearch => self.process_enter_search(),
            Command::InputChar(c) => self.process_input_char(c),
            Command::ClearSearch => self.process_clear_search(),
            Command::RemoveChar => self.process_remove_char(),
            Command::Enter => self.process_enter(),
        }
    }

    fn process_enter(&mut self){
        if self.is_searching{
            self.is_searching = false;
        }

    }
    fn process_next_command(&mut self){
        let new_index = (self.selected_index.saturating_add(1)).min(self.visible_rows-1);
        let _= self.select_satellite(new_index);
    }

    fn process_next_page_command(&mut self){
        let nb_page = self.calculate_page_count();
        self.page_index = self.page_index.saturating_add(1).min(nb_page.saturating_sub(1)); // index pages : 0..nb_pages-1
        let _= self.select_satellite(self.selected_index); // Update the satellite at the current index
    }

    fn process_previous_page_command(&mut self){
        self.page_index = self.page_index.saturating_sub(1).max(0);
        let _= self.select_satellite(self.selected_index); // Update the satellite at the current index
    }

    fn process_previous_command(&mut self){
        let new_index = 0.max(self.selected_index.saturating_sub(1));
        let _= self.select_satellite(new_index);
    }

    fn process_enter_search(&mut self){
        self.is_searching = true;
    }


    fn process_input_char(&mut self, c:char){
        if self.is_searching{
            self.search_query.push(c);
            self.update_satellites();
            let _= self.update_history(45);
        }
    }

    fn process_clear_search(&mut self){

        self.search_query.clear();
        self.is_searching = false;
        self.update_satellites();
        let _= self.update_history(45);

    }

    fn process_remove_char(&mut self){
        if self.is_searching{
            match self.search_query.pop(){
                Some(_c) => {
                    self.update_satellites();
                    let _= self.update_history(45);
                }
                _ => {
                    self.process_clear_search();
                }
            }
        }
    }
}