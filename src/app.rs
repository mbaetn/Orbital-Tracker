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
}

pub struct SatelliteState{
    name:String,
    pos: LongLatCoordinates
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
        let res = Self{selected_index, page_index, satellites, history, should_quit, visible_rows};

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

            let start_index = self.page_index*self.visible_rows;
            let tracked_positions: Vec<SatelliteState> = self.compute_positions(Utc::now(), start_index, start_index + self.visible_rows);
            let max_index = tracked_positions.len().min(self.satellites.len()).saturating_sub(1);

            // Case when the window size is reduced
            if self.selected_index > max_index{
                self.select_satellite(self.selected_index.min(max_index))?;
            }


            // Update UI
            ui.set_selected_row(self.selected_index);
            let nb_pages = self.calculate_page_count();

            ui.draw(tracked_positions, &self.history, self.page_index, nb_pages)?;


            if poll(Duration::from_millis(1000))? { // If nothing happens during dt, skip that part and refresh position
                if let Event::Key(key_event) = read()? {
                    self.handle_command(key_event);
                }
            }


        }
        ui.terminate()?;
        Ok(())

    }

    /// Satellite index in `satellites`
    fn active_satellite_index(&self) -> usize {
        if self.satellites.is_empty() {
            return 0;
        }
        let page_size = self.visible_rows.max(1);
        let raw = self.page_index.saturating_mul(page_size).saturating_add(self.selected_index);
        raw.min(self.satellites.len() - 1)
    }

    fn calculate_page_count(&self)->usize{
        self.satellites.len().saturating_div(self.visible_rows)
    }

    fn select_satellite(&mut self, index:usize) -> Result<(), Box <dyn Error>>{
        self.selected_index = index;
        self.update_history(45)?;
        Ok(())
    }

    /// Compute positions of satellites between `start_index` and `end_index`
    fn compute_positions(&self, now: DateTime<Utc>, start_index:usize, end_index:usize) -> Vec<SatelliteState>{

        let end = end_index.min(self.satellites.len());
        self.satellites[start_index..end]
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

    fn update_history(&mut self, timelaps:i64) -> Result<(), Box <dyn Error>>{
        let mut history = Vec::new();
        let active_index = self.active_satellite_index();
        let target = &self.satellites[active_index];
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
            _ => (),
        }
    }

    fn process_next_command(&mut self){
        let new_index = (self.selected_index.saturating_add(1)).min(self.visible_rows-1);
        let _= self.select_satellite(new_index);
    }

    fn process_next_page_command(&mut self){
        let nb_page = self.calculate_page_count();
        self.page_index = self.page_index.saturating_add(1).min(nb_page);
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
}