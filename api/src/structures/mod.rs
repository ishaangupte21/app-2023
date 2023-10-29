// This file defines all structures to be used within the application.
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CollegeStruct {
    ipedsid: String,
    name: String,
    address: String,
    city: String,
    state: String,
    zip: String,
    geo_point_2d: CollegeCoord,
    naics_desc: String
}

#[derive(Deserialize, Serialize)]
pub struct CollegeCoord {
    pub lon: f64,
    pub lat: f64,
}
