// This file defines all structures to be used within the application.
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CollegeStruct {
    pub ipedsid: String,
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub geo_point_2d: CollegeCoord,
    pub naics_desc: String,
}

#[derive(Deserialize, Serialize, std::clone::Clone)]
pub struct CollegeCoord {
    pub lon: f64,
    pub lat: f64,
}
