use crate::schema::sensor_veri;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct SensorData {
    pub id: i32,
    pub sicaklik: f32,
    pub nem: f32,
    pub timestamp: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "sensor_veri"]
pub struct NewSensorData {
    pub sicaklik: f32,
    pub nem: f32,
    pub timestamp: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SensorPostData {
    pub sicaklik: f32,
    pub nem: f32,
}
