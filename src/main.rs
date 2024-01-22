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

// Just returns "ok" to show that communication with the firmware is working.
#[get("/hello")]
fn index(streamobj: &State<RwLock<UnixStream>>) -> String {
    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(b"0\n\n");
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

// Gets current state. Example result:
// tgHeatLevelR = 0
// tgHeatLevelL = 0
// heatTimeL = 0
// heatLevelL = -100
// heatTimeR = 0
// heatLevelR = -100
// sensorLabel = null
// waterLevel = true
// priming = false
// settings = "BF61760162676C190190626772190190626C6200FF"
#[get("/variables")]
fn variables(streamobj: &State<RwLock<UnixStream>>) -> String {
    let mut stream = streamobj.write().unwrap();
    let _ = stream.write(b"14\n\n");
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

// Example CBOR: a462706c18326264751902586274741a65af6af862706966646f75626c65
// pl: Vibration intensity percentage
// pi: Vibration pattern ("double" (heavy) or "rise" (gentle))?
// du: Duration in seconds?
// tt: Timestamp in unix epoch for alarm
// Presumably thermal alarm is controlled with the temperature commands
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

// Example CBOR: a1626c6200, a1626c621837. Controls light intensity.
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

// Takes an integer number of seconds, presumably until the heat ends, e.g. 7200.
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

// Takes a signed integer number. May represent tenths of degrees of heating/cooling. e.g. -40 = -4°C.
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

// Takes a boolean string. Unclear what true/false mean exactly, maybe on/off?
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
