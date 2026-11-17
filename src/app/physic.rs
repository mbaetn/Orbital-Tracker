use sgp4::{Constants, Elements, MinutesSinceEpoch, Prediction};
use std::error::Error;
use super::parser::TleData; 
use chrono::{DateTime, Utc};

/// SGP4 Propagator, derives positions and velocities for a given satellite using [TleData] structure
pub struct SGP4Propagator {
    constants: Constants,
    epoch: DateTime<Utc>,
}


impl SGP4Propagator {
    
    /// Initializes propagator 
    /// # Argument
    /// `target` : satellite informations obtained with an API call
    pub fn new(target: &TleData) -> Result<Self, Box<dyn Error>> {
        let elt = Elements::from_tle(
            Some(target.name.to_owned()), 
            target.line_1.as_bytes(), 
            target.line_2.as_bytes()
        )?;
        let epoch = elt.datetime;
        let epoch_utc = epoch.and_utc();

        let constants = Constants::from_elements(&elt)?;
        
        Ok(SGP4Propagator { constants, epoch:epoch_utc}) 
    }

    /// Calculate the SGP4 position and velocity current predictions 
    pub fn get_position(&self, time:DateTime<Utc>)-> Result<Prediction,sgp4::Error>{
        let delta = time.signed_duration_since(self.epoch);
        let minute_since_epoch = delta.num_milliseconds() as f64 / 60_000.0; // Take milliseconds to keep decimal precision


        self.constants.propagate(MinutesSinceEpoch(minute_since_epoch))
    }
}