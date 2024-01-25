#[macro_use]
extern crate rocket;
extern crate rustc_serialize;

use cbor::{Cbor, Decoder, Encoder, ToCbor};
use log::{info, warn};
use rocket::State;
use rustc_serialize::{json::Json, Encodable};
use std::{
    collections::HashMap,
    io::{BufReader, BufWriter, Read, Write},
    net::TcpListener,
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

// Just returns "ok" to show that communication with the firmware is working.
#[get("/hello")]
fn index(streamobj: &State<Arc<RwLock<Option<UnixStream>>>>) -> String {
    if streamobj.read().unwrap().is_none() {
        return "not connected".to_string();
    }
    let mut streamoption = streamobj.write().unwrap();
    let stream = streamoption.as_mut().unwrap();
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
fn variables(streamobj: &State<Arc<RwLock<Option<UnixStream>>>>) -> String {
    if streamobj.read().unwrap().is_none() {
        return "not connected".to_string();
    }
    let mut streamoption = streamobj.write().unwrap();
    let stream = streamoption.as_mut().unwrap();
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
fn alarm(side: &str, data: &str, streamobj: &State<Arc<RwLock<Option<UnixStream>>>>) -> String {
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

    if streamobj.read().unwrap().is_none() {
        return "not connected".to_string();
    }
    let mut streamoption = streamobj.write().unwrap();
    let stream = streamoption.as_mut().unwrap();
    let _ = stream.write(format!("{}\n{}\n\n", command, serializeddata).as_bytes());
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[post("/alarm-clear")]
fn alarm_clear(streamobj: &State<Arc<RwLock<Option<UnixStream>>>>) -> String {
    if streamobj.read().unwrap().is_none() {
        return "not connected".to_string();
    }
    let mut streamoption = streamobj.write().unwrap();
    let stream = streamoption.as_mut().unwrap();
    let _ = stream.write(b"16\n\n");
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

// Example CBOR: a1626c6200, a1626c621837. Controls light intensity.
#[post("/settings", data = "<data>")]
fn settings(data: &str, streamobj: &State<Arc<RwLock<Option<UnixStream>>>>) -> String {
    let jsondata = Json::from_str(data).unwrap();
    let cbordata = jsondata.to_cbor();
    let mut cborencoder = Encoder::from_memory();
    cbordata.encode(&mut cborencoder).unwrap();
    let serializeddata = hex::encode(cborencoder.as_bytes());

    if streamobj.read().unwrap().is_none() {
        return "not connected".to_string();
    }
    let mut streamoption = streamobj.write().unwrap();
    let stream = streamoption.as_mut().unwrap();
    let _ = stream.write(format!("8\n{}\n\n", serializeddata).as_bytes());
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

// Takes an integer number of seconds, presumably until the heat ends, e.g. 7200.
#[post("/temperature-duration/<side>", data = "<data>")]
fn temperature_duration(
    side: &str,
    data: &str,
    streamobj: &State<Arc<RwLock<Option<UnixStream>>>>,
) -> String {
    let command = match side {
        "left" => 9,
        "right" => 10,
        _ => {
            panic!("Invalid side requested")
        }
    };

    if streamobj.read().unwrap().is_none() {
        return "not connected".to_string();
    }
    let mut streamoption = streamobj.write().unwrap();
    let stream = streamoption.as_mut().unwrap();
    let _ = stream.write(format!("{}\n{}\n\n", command, data).as_bytes());
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

// Takes a signed integer number. May represent tenths of degrees of heating/cooling. e.g. -40 = -4°C.
#[post("/temperature/<side>", data = "<data>")]
fn temperature(
    side: &str,
    data: &str,
    streamobj: &State<Arc<RwLock<Option<UnixStream>>>>,
) -> String {
    let command = match side {
        "left" => 11,
        "right" => 12,
        _ => {
            panic!("Invalid side requested")
        }
    };

    if streamobj.read().unwrap().is_none() {
        return "not connected".to_string();
    }
    let mut streamoption = streamobj.write().unwrap();
    let stream = streamoption.as_mut().unwrap();
    let _ = stream.write(format!("{}\n{}\n\n", command, data).as_bytes());
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

// Takes a boolean string. Unclear what true/false mean exactly, maybe on/off?
#[post("/prime")]
fn prime(streamobj: &State<Arc<RwLock<Option<UnixStream>>>>) -> String {
    if streamobj.read().unwrap().is_none() {
        return "not connected".to_string();
    }
    let mut streamoption = streamobj.write().unwrap();
    let stream = streamoption.as_mut().unwrap();
    let _ = stream.write(b"13\n\n");
    let _ = stream.set_read_timeout(Some(Duration::new(0, 50000000)));
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    return result;
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct StreamItem {
    part: String,
    proto: String,
    id: Option<u32>,
    version: Option<String>,
    dev: Option<String>,
    stream: Option<Vec<u8>>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct BatchItem {
    seq: u32,
    data: Vec<u8>,
}

#[rocket::main]
async fn main() {
    env_logger::init();

    let stream = Arc::new(RwLock::<Option<UnixStream>>::new(None));

    let streamcopy = stream.clone();
    thread::spawn(move || {
        let listener = match UnixListener::bind("/deviceinfo/dac.sock") {
            Ok(listener) => listener,
            Err(error) => {
                panic!("Failed to listen {:?}", error)
            }
        };
        for newstream in listener.incoming() {
            match newstream {
                Ok(newstream) => {
                    info!("New UNIX socket connection");
                    let _ = streamcopy.write().unwrap().insert(newstream);
                }
                Err(_) => continue,
            }
        }
    });

    thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:1337").unwrap();
        for stream in listener.incoming() {
            let stream = match stream {
                Err(_) => continue,
                Ok(stream) => stream,
            };
            thread::spawn(move || {
                info!("Incoming TCP connection");
                let _ = stream.set_read_timeout(Some(Duration::new(60, 0)));
                let reader = BufReader::new(&stream);
                let mut writer = BufWriter::new(&stream);

                let mut d = Decoder::from_reader(reader);
                for item in d.decode::<StreamItem>() {
                    let item = item.expect("failed to parse CBOR item");
                    match item.part.as_str() {
                        "session" => {
                            info!("Session started: {:?}", item);
                            let mut encoder = Encoder::from_memory();
                            let res = StreamItem {
                                part: "session".into(),
                                proto: "raw".into(),
                                id: None,
                                version: None,
                                dev: None,
                                stream: None,
                            };
                            info!("Responding with {:?}", res);
                            let _ = res.encode(&mut encoder);
                            trace!("Response bytes {}", hex::encode(encoder.as_bytes()));
                            let _ = writer.write(encoder.as_bytes());
                            let _ = writer.flush();
                        },
                        "batch" => {
                            let id = match item.id {
                                Some(id) => id,
                                None => {warn!("no id was present for batch"); continue;}
                            };
                            info!("Received batch {}", id);
                            let mut encoder = Encoder::from_memory();
                            let _ = StreamItem {
                                id: Some(id),
                                proto: "raw".into(),
                                part: "batch".into(),
                                dev: None,
                                stream: None,
                                version: None,
                            }
                            .encode(&mut encoder);
                            let _ = writer.write(encoder.as_bytes());
                            let _ = writer.flush();

                            let datastream = match item.stream {
                                Some(stream) => stream,
                                None => {warn!("no stream in batch"); continue;},
                            };
                            let mut d = Decoder::from_bytes(datastream);
                            for item in d.decode::<BatchItem>() {
                                let item = match item {
                                    Ok(item) => item,
                                    Err(error) => {
                                        warn!("failed to parse batch item: {:?}", error);
                                        continue;
                                    },
                                };
                                //info!("Batch item: {:?}", item);

                                let mut d = Decoder::from_bytes(item.data);
                                let seq = item.seq;
                                for item in d.items() {
                                    let item = match item {
                                        Ok(item) => item,
                                        Err(error) => {
                                            warn!("failed to parse batch {} datum: {:?}", seq, error);
                                            continue;
                                        },
                                    };
                                    info!("Batch item {} datum: {:?}", seq, item);
                                }
                            }
                        }
                        _ => {
                            warn!("Unrecognized part {:?}", item.part);
                            continue;
                        }
                    }
                }
            });
        }
    });

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
        .manage(stream)
        .launch()
        .await
    {
        Ok(ignite) => ignite,
        Err(error) => {
            panic!("Failed to start rocket {:?}", error)
        }
    };
}
