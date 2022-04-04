use crate::schema::sensor_data;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct SensorData {
    pub id: i32,
    pub sicaklik: f64,
    pub nem: f64,
    pub timestamp: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "sensor_data"]
pub struct NewSensorData {
    pub sicaklik: f64,
    pub nem: f64,
    pub timestamp: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SensorPostData {
    pub sicaklik: f64,
    pub nem: f64,
}
