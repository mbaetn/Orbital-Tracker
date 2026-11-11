use chrono::{DateTime, Utc};


pub struct LongLatCoordinates{
    pub longitude:f64,
    pub latitude:f64
}


impl LongLatCoordinates{
    pub fn from_cartesian(cartesian: [f64;3], time:DateTime<Utc>) -> Self{
        let x = cartesian[0];
        let y = cartesian[1];
        let z = cartesian[2];

        let xy_dist = (x*x + y*y).sqrt();

        let latitude= z.atan2(xy_dist).to_degrees();
        let lon_teme_rad = y.atan2(x);
        // Angle of Greenwhich GMST
        let gmst_rad = Self::calculate_gmst(time);
        let mut longitude = (lon_teme_rad - gmst_rad).to_degrees();

        // Normalization
        longitude = (longitude + 540.0) % 360.0 - 180.0;
        
        Self {longitude, latitude}
    }

    /// Calculate Greenwich Mean Sideral Time (GMST) for a given date
    fn calculate_gmst(date: DateTime<Utc>) -> f64 {
        // Julian day
        let jd = date.timestamp() as f64 / 86400.0 + 2440587.5;
        let t = (jd - 2451545.0) / 36525.0; // Julian century since J2000.0
        
        // USNO / Vallado formula
        let mut gmst_sec = 67310.54841 
            + (876600.0 * 3600.0 + 8640184.812866) * t 
            + 0.093104 * t * t 
            - 6.2e-6 * t * t * t;
        
        // Modulo number of seconds in a day 86400
        gmst_sec = gmst_sec % 86400.0;
        if gmst_sec < 0.0 {
            gmst_sec += 86400.0;
        }
        
        
        gmst_sec * (std::f64::consts::PI / 43200.0)
    }
}