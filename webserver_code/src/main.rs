#[macro_use]
extern crate rocket;

use rand::{seq::IteratorRandom, Rng, RngCore};
use rocket::{
    fs::{relative, FileServer},
    http::Status,
    response::Redirect,
    serde::json::Json,
    State,
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
#[derive(Debug, Serialize, Deserialize)]
struct SetSensor {
    id: u32,
    watering_time: u16,
    backoff_time: u16,
    threshold: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorCodeConfig {
    error_codes: Vec<u16>,
    error_count_range: Range,
}
#[get("/")]
fn redirect_to_static() -> Redirect {
    Redirect::permanent(uri!("/static"))
}
#[post("/sensor", data = "<sensor>")]
fn set_sensor(mock_sensors: &State<Vec<PlantObject>>, sensor: Json<SetSensor>) -> Status {
    if let Some(to_edit) = mock_sensors.iter().find(|s| s.id == sensor.id) {
        to_edit
            .watering_time
            .store(sensor.watering_time, std::sync::atomic::Ordering::Relaxed);
        to_edit
            .backoff_time
            .store(sensor.backoff_time, std::sync::atomic::Ordering::Relaxed);
        to_edit
            .threshold
            .store(sensor.threshold, std::sync::atomic::Ordering::Relaxed);
        Status::Ok
    } else {
        Status::NotFound
    }
}
fn get_random_from_range<T: RngCore>(rng: &mut T, range: Range) -> i32 {
    rng.random_range(range.min..=range.max)
}
#[get("/sensor/<id>")]
fn get_sensor(mock_sensors: &State<Vec<PlantObject>>, id: u32) -> Json<Option<ReturnSensor>> {
    let mut rng = rand::rng();
    Json(mock_sensors.iter().find_map(|sensor| {
        if id == sensor.id {
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
#[get("/errors")]
fn get_errors(error_config: &State<ErrorCodeConfig>) -> Json<Vec<u16>> {
    let mut rng = rand::rng();
    let count = get_random_from_range(&mut rng, error_config.error_count_range)
        .min(error_config.error_codes.len() as i32);

    Json(
        error_config
            .error_codes
            .iter()
            .cloned()
            .choose_multiple(&mut rng, count as usize),
    )
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

    let mut file = match File::open("errors.json") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("failed to open file: {}", e);
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
    let error_config: ErrorCodeConfig = match serde_json::from_str(&data) {
        Ok(objects) => objects,
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            panic!("abort3");
        }
    };
    rocket::build()
        .manage(objects)
        .manage(error_config)
        .mount(
            "/",
            routes![
                redirect_to_static,
                get_sensor_list,
                get_sensor,
                set_sensor,
                get_errors
            ],
        )
        .mount("/static", FileServer::from("static/"))
}
