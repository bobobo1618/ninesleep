#[macro_use]
extern crate rocket;

use rocket::State;
use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::RwLock,
    time::Duration,
};
use cbor::{Encoder, ToCbor};
use rustc_serialize::{json::Json, Encodable};

#[get("/hello")]
fn index(streamobj: &State<RwLock<UnixStream>>) -> String {
    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(b"0\n\n");
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[get("/variables")]
fn variables(streamobj: &State<RwLock<UnixStream>>) -> String {
    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(b"14\n\n");
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[post("/alarm/<side>", data = "<data>")]
fn alarm(side: &str, data: &str, streamobj: &State<RwLock<UnixStream>>) -> String {
    let command = match side {
        "left" => 5,
        "right" => 6,
        _ => {
            panic!("Invalid side requested")
        }
    };

    let jsondata = Json::from_str(data).unwrap();
    let cbordata = jsondata.to_cbor();
    let mut cborencoder = Encoder::from_memory();
    cbordata.encode(&mut cborencoder).unwrap();
    let serializeddata = hex::encode(cborencoder.as_bytes());

    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(format!("{}\n{}\n\n", command, serializeddata).as_bytes());
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[post("/alarm-clear")]
fn alarm_clear(streamobj: &State<RwLock<UnixStream>>) -> String {
    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(b"16\n\n");
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[post("/settings", data = "<data>")]
fn settings(data: &str, streamobj: &State<RwLock<UnixStream>>) -> String {
    let jsondata = Json::from_str(data).unwrap();
    let cbordata = jsondata.to_cbor();
    let mut cborencoder = Encoder::from_memory();
    cbordata.encode(&mut cborencoder).unwrap();
    let serializeddata = hex::encode(cborencoder.as_bytes());

    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(format!("8\n{}\n\n", serializeddata).as_bytes());
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[post("/temperature-duration/<side>", data = "<data>")]
fn temperature_duration(side: &str, data: &str, streamobj: &State<RwLock<UnixStream>>) -> String {
    let command = match side {
        "left" => 9,
        "right" => 10,
        _ => {
            panic!("Invalid side requested")
        }
    };

    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(format!("{}\n{}\n\n", command, data).as_bytes());
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[post("/temperature/<side>", data = "<data>")]
fn temperature(side: &str, data: &str, streamobj: &State<RwLock<UnixStream>>) -> String {
    let command = match side {
        "left" => 11,
        "right" => 12,
        _ => {
            panic!("Invalid side requested")
        }
    };

    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(format!("{}\n{}\n\n", command, data).as_bytes());
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[post("/prime")]
fn prime(streamobj: &State<RwLock<UnixStream>>) -> String {
    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(b"13\n\n");
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[rocket::main]
async fn main() {
    let listener = match UnixListener::bind("/deviceinfo/dac.sock") {
        Ok(listener) => listener,
        Err(error) => {
            panic!("Failed to listen {:?}", error)
        }
    };

    let stream = match listener.incoming().next() {
        Some(val) => match val {
            Ok(stream) => stream,
            Err(error) => {
                panic!("Failed to get connection {:?}", error)
            }
        },
        None => {
            panic!("Failed to get connection")
        }
    };

    println!("Client connected, starting rocket.");

    let _rocket = match rocket::build()
        .mount(
            "/",
            routes![
                index,
                variables,
                alarm,
                alarm_clear,
                settings,
                temperature,
                temperature_duration,
                prime
            ],
        )
        .manage(RwLock::new(stream))
        .launch()
        .await
    {
        Ok(ignite) => ignite,
        Err(error) => {
            panic!("Failed to start rocket {:?}", error)
        }
    };
}
