// This file defines all structures to be used within the application.
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CollegeStruct {
    objectid: String,
    name: String,
    address: String,
    city: String,
    state: String,
    zip: String,
}
