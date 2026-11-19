use super::TleData;
use super::SGP4Propagator;

pub struct SatelliteEntity{
    tle : TleData,
    propagator : SGP4Propagator
}

impl SatelliteEntity{
    pub fn new(tle: TleData, propagator: SGP4Propagator) -> Self{
        Self {tle, propagator}
    }
    pub fn get_propagator(&self) -> &SGP4Propagator{
        &self.propagator
    }
    pub fn get_tle(&self) -> &TleData{
        &self.tle
    }
}