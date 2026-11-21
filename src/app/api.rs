use std::error::Error;
use serde::Deserialize;
use std::fs;
use ureq;
use std::io::Write;
use std::io;


/// Structure to keep satellite information using Two Line Element format
#[derive(Debug, Deserialize, Clone)]
pub struct TleData{
    /// ID of the satellite
    // #[serde(rename="NORAD_CAT_ID")]
    // pub norad_id: String,
    /// Common name
    #[serde(rename="name")]
    pub name: Option<String>,
    /// Line 1 : contains identification informations
    #[serde(rename="tle1")]
    pub line_1: String,
    /// Line 2 : contains pure geometrical informations
    #[serde(rename = "tle2")]
    pub line_2: String,
}


pub struct ApiClient{
    base_url: String,
    api_key: String,
}

impl ApiClient{
    pub fn new(base_url: &str, api_key: &str) -> Self{
        ApiClient { base_url: base_url.to_string(), api_key: api_key.to_string() }
    }

    pub fn get_satellites_tles(&self)->Result<Vec<TleData>, Box<dyn Error>>{

        let cache_file = "tle_cache.json";
        let mut needs_refresh = true; // See if we need to refresh Tle Data

        if let Ok(metadata) = fs::metadata(cache_file){
            io::stdout().flush().unwrap();
            if let Ok(modified_time) = metadata.modified(){
                if let Ok(elapsed) = modified_time.elapsed(){
                    // 4 hours : 14 400s
                    if elapsed.as_secs() <14_400{
                        needs_refresh = false;
                    }
                }
            }
        }

        let response = if needs_refresh{
            let fetched_data = ureq::get(&self.base_url)
            .header("X-API-Key", &self.api_key)
            .call()?
            .body_mut()
            .read_to_string()?;

            fs::write(cache_file, &fetched_data)?;
            fetched_data
        } else{
            fs::read_to_string(cache_file)?
        };


        let mut satellites: Vec<TleData> = serde_json::from_str(&response)?;
        satellites.retain(|target| target.name.is_some());
        Ok(satellites)
    }
}
