// External imports
use std::{env};
use chrono::Utc;
use dotenv::dotenv;
use ureq;
use crossterm::{
    event::{poll, read, Event}
};
use std::time::Duration;

// Local imports
mod parser;
use parser::{TleData, ApiClient};
mod physic;
use physic::SGP4Propagator;
mod interface;
use interface::{Interface, System};
use interface::coordinates::LongLatCoordinates;


fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();
    let api_key = env::var("API_KEY").expect("No API_KEY found");
    let base_url = "https://api.keeptrack.space/v4/sats/25544";

    let api =  ApiClient::new(base_url, &api_key);
    let satellites = api.get_satellites_informations()?;

    
    // println!("Number of satellites : {}", satellites.len());
    if let Some(target) = satellites.first(){
        let propagator : SGP4Propagator = SGP4Propagator::new(&target).unwrap();
        
        let mut terminal = Interface::new()?;
        let mut history: Vec<(f64, f64)> = Vec::new(); // History of positions 
        let now = Utc::now();

        // ISS Period is 90 minutes
        for i in -45..=45 {
            let current_step_time = now + chrono::Duration::try_minutes(i).unwrap();
            
        
            if let Ok(prediction) = propagator.get_position(current_step_time) {
                let coords = LongLatCoordinates::from_cartesian(prediction.position, current_step_time);
                history.push((coords.longitude, coords.latitude));
            }

        }
        // Main loop
        loop{

            let predictions = propagator.get_position(Utc::now())?;
            let cartesians = predictions.position;


            // let text = format!("X:{:.2} ; Y:{:.2} ; Z:{:.2}", positions[0], positions[1], positions[2]);
            let pos = LongLatCoordinates::from_cartesian(cartesians, Utc::now());
            let text = format!("Latitude : {:.2} ; Longitude : {:.2}", pos.latitude, pos.longitude);
            // let pos = LongLatCoordinates{latitude:45.0, longitude:10.0};
            terminal.draw(pos, &text, &history)?;
            // terminal.draw(&target.norad_id)?;
    
            if poll(Duration::from_millis(1000))? { // If nothing happens during one second, skip that part and refresh position
                if let Event::Key(key_event) = read()? {
                    if let Ok(cmd) = System::try_from(key_event){
                        match cmd{
                            System::Quit => break
                        }
                    }
                }
            }
        }

        Interface::terminate()?;
        
    }


    Ok(())


}

