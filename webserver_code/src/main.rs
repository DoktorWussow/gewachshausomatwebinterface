#[macro_use]
extern crate rocket;

use rand::{Rng, RngCore};
use rocket::{
    State,
    fs::{FileServer, relative},
    response::Redirect,
    serde::json::Json,
};

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::{fs::File, sync::atomic::AtomicU16};

// Define a generic struct for ranges
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct Range {
    min: i32,
    max: i32,
}

// Define a struct for the environmental conditions using the generic Range struct
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct EnvConditions {
    temperature_range: Range,
    humidity_range: Range,
}

// Define the main struct for each object in the JSON array
#[derive(Debug, Serialize, Deserialize)]
struct PlantObject {
    id: u32,
    watering_time: AtomicU16,
    backoff_time: AtomicU16,
    threshold: AtomicU16,
    moisture_range: Range,
    env_conditions: Option<EnvConditions>, // This field is optional
}
#[derive(Debug, Serialize, Deserialize)]
struct ReturnSensor {
    id: u32,
    watering_time: u16,
    backoff_time: u16,
    threshold: u16,
    moisture: i32,
    temperature: Option<i32>,
    humidity: Option<i32>,
}
#[get("/")]
fn redirect_to_static() -> Redirect {
    Redirect::permanent(uri!("/static"))
}
fn get_random_from_range<T: RngCore>(rng: &mut T, range: Range) -> i32 {
    rng.random_range(range.min..=range.max)
}
#[get("/sensor/<id>")]
fn get_sensor(mock_sensors: &State<Vec<PlantObject>>, id: u32) -> Json<Option<ReturnSensor>> {
    let mut rng = rand::rng();
    Json(mock_sensors.iter().find_map(|sensor| {
        if (id == sensor.id) {
            let moisture = get_random_from_range(&mut rng, sensor.moisture_range);
            let (temperature, humidity) = if let Some(env_cond) = sensor.env_conditions {
                (
                    Some(get_random_from_range(&mut rng, env_cond.temperature_range)),
                    Some(get_random_from_range(&mut rng, env_cond.humidity_range)),
                )
            } else {
                (None, None)
            };
            Some(ReturnSensor {
                id: sensor.id,
                watering_time: sensor
                    .watering_time
                    .load(std::sync::atomic::Ordering::Relaxed),
                backoff_time: sensor
                    .backoff_time
                    .load(std::sync::atomic::Ordering::Relaxed),
                threshold: sensor.threshold.load(std::sync::atomic::Ordering::Relaxed),
                moisture: moisture,
                temperature: temperature,
                humidity: humidity,
            })
        } else {
            None
        }
    }))
}
#[get("/sensors")]
fn get_sensor_list(mock_sensors: &State<Vec<PlantObject>>) -> Json<Vec<u32>> {
    Json(mock_sensors.iter().map(|sensor| sensor.id).collect())
}

#[launch]
fn rocket() -> _ {
    // Open the file
    let mut file = match File::open("config.json") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            panic!("abort")
        }
    };

    // Read the file contents into a string
    let mut data = String::new();
    if let Err(e) = file.read_to_string(&mut data) {
        eprintln!("Failed to read file: {}", e);
        panic!("abort2");
    }

    // Parse the JSON data into a vector of PlantObject instances
    let objects: Vec<PlantObject> = match serde_json::from_str(&data) {
        Ok(objects) => objects,
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            panic!("abort3");
        }
    };

    // Print the deserialized objects
    for object in &objects {
        println!("{:#?}", object);
    }
    rocket::build()
        .manage(objects)
        .mount(
            "/",
            routes![redirect_to_static, get_sensor_list, get_sensor],
        )
        .mount("/static", FileServer::from(relative!("static")))
}
