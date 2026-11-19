use chrono::Utc;
use std::{env, time::Duration};
use std::error::Error;
use dotenv::dotenv;
use crossterm::{
    event::{poll, read, Event}
};


mod api;
use api::ApiClient;
use api::TleData;
mod physic;
use physic::SGP4Propagator;
use crate::app::interface::coordinates::LongLatCoordinates;
mod interface;
use interface::System;
use interface::Interface;
mod satellite_entity;
use satellite_entity::SatelliteEntity;

pub struct App{
    satellites: Vec<SatelliteEntity>,
    history: Vec<LongLatCoordinates>,
    should_quit: bool,
}


impl App{

    pub fn init()-> Result<Self, Box<dyn Error>>{
        dotenv().ok();
        let api_key = env::var("API_KEY").expect("No API_KEY found");
        let base_url = "https://api.keeptrack.space/v4/sats";

        let api =  ApiClient::new(base_url, &api_key);
        let tle_satellites:Vec<TleData> = api.get_satellites_informations()?;

        let satellites: Vec<SatelliteEntity> =  tle_satellites.into_iter().filter_map(|target| Self::to_satellite_entity(target).ok().flatten()).collect();

        let history = Vec::new();
        let should_quit = false;
        let res = Self{satellites, history, should_quit};

        Ok(res)


    }

    fn to_satellite_entity(target: TleData) -> Result<Option<SatelliteEntity>, Box<dyn Error>>{
        let propagator = SGP4Propagator::new(&target)?;
        Ok(Some(SatelliteEntity::new(target, propagator)))

    }
    pub fn run(&mut self) -> Result<(), Box<dyn Error>>{

        let mut ui = Interface::new()?;
        self.history = self.update_history(45)?;

        while !self.should_quit{
            let target = &self.satellites[0];
            let propagator = target.get_propagator();
            let predictions = propagator.get_position(Utc::now())?;
            let cartesians = predictions.position;


            // let text = format!("X:{:.2} ; Y:{:.2} ; Z:{:.2}", positions[0], positions[1], positions[2]);
            let pos = LongLatCoordinates::from_cartesian(cartesians, Utc::now());
            // let text = "🛰️";
            ui.draw(pos, &target.get_tle(), &self.history)?;
    
            if poll(Duration::from_millis(100))? { // If nothing happens during one second, skip that part and refresh position
                if let Event::Key(key_event) = read()? {
                    if let Ok(cmd) = System::try_from(key_event){
                        match cmd{
                            System::Quit => self.should_quit = true,
                        }
                    }
                }
            }
        }
        ui.terminate()?;
        Ok(())

    }

    fn update_history(&self, timelaps:i64) -> Result<Vec<LongLatCoordinates>, Box <dyn Error>>{
        let mut history = Vec::new();
        let target = &self.satellites[0];
        let propagator = target.get_propagator();
        let now = Utc::now();
        for i in -timelaps..timelaps{
            let current_step_time = now + chrono::Duration::try_minutes(i).unwrap();   
            if let Ok(prediction) = propagator.get_position(current_step_time) {
                let coords = LongLatCoordinates::from_cartesian(prediction.position, current_step_time);
                history.push(coords);
            }
        }

        Ok(history)
    }
}