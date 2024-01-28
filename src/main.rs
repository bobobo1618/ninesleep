#[macro_use]
extern crate rocket;
extern crate rustc_serialize;

use cbor::{Encoder, ToCbor};
use chrono::{serde::ts_seconds, DateTime, Utc};
use log::{info, warn};
use rocket::State;
use rustc_serialize::{json::Json, Encodable};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};
use serde_bytes;

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

#[derive(Debug, Serialize, Deserialize)]
struct AlarmSettings {
    pl: u8,
    du: u16,
    pi: String,
    tt: u64,
}

// Example CBOR: a462706c18326264751902586274741a65af6af862706966646f75626c65
// pl: Vibration intensity percentage
// pi: Vibration pattern ("double" (heavy) or "rise" (gentle))?
// du: Duration in seconds?
// tt: Timestamp in unix epoch for alarm
// Presumably thermal alarm is controlled with the temperature commands
#[post("/alarm/<side>", data = "<data>", format = "json")]
fn alarm(side: &str, data: rocket::serde::json::Json<AlarmSettings>, streamobj: &State<Arc<RwLock<Option<UnixStream>>>>) -> String {
    let command = match side {
        "left" => 5,
        "right" => 6,
        _ => {
            panic!("Invalid side requested")
        }
    };
    
    let data = data.into_inner();
    let mut bincbor = Vec::<u8>::new();
    ciborium::into_writer(&data, &mut bincbor).unwrap();
    let serializeddata = hex::encode(bincbor);

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

#[derive(Debug, Serialize, Deserialize)]
struct StreamItem {
    part: String,
    proto: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BatchItem {
    seq: u32,
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CapSenseSide {
    status: String,
    cen: u16,
    #[serde(rename = "in")]
    in_: u16,
    out: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct CapSense {
    #[serde(with = "ts_seconds")]
    ts: DateTime<Utc>,
    left: CapSenseSide,
    right: CapSenseSide,
}

#[derive(Debug, Serialize, Deserialize)]
struct PiezoDual {
    #[serde(with = "ts_seconds")]
    ts: DateTime<Utc>,
    adc: u8,
    freq: u16,
    gain: u16,
    #[serde(with = "serde_bytes")]
    left1: Vec<u8>,
    #[serde(with = "serde_bytes")]
    left2: Vec<u8>,
    #[serde(with = "serde_bytes")]
    right1: Vec<u8>,
    #[serde(with = "serde_bytes")]
    right2: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BedTempSide {
    cen: u16,
    #[serde(rename = "in")]
    in_: u16,
    out: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct BedTemp {
    #[serde(with = "ts_seconds")]
    ts: DateTime<Utc>,
    mcu: u16,
    amb: u16,
    hu: u16,
    left: BedTempSide,
    right: BedTempSide,
}

#[derive(Debug, Serialize, Deserialize)]
struct BatchItemLog {
    #[serde(with = "ts_seconds")]
    ts: DateTime<Utc>,
    msg: String,
    level: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrzTemp {
    #[serde(with = "ts_seconds")]
    ts: DateTime<Utc>,
    amb: u16,
    hs: u16,
    left: u16,
    right: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum BatchItemData {
    #[serde(rename = "capSense")]
    CapSense(CapSense),
    #[serde(rename = "piezo-dual")]
    PiezoDual(PiezoDual),
    #[serde(rename = "bedTemp")]
    BedTemp(BedTemp),
    #[serde(rename = "log")]
    BatchItemLog(BatchItemLog),
    #[serde(rename = "frzTemp")]
    FrzTemp(FrzTemp),
}

fn handle_session(item: StreamItem, writer: &mut dyn Write) {
    info!(
        "Session started for device {}",
        item.dev.expect("expected device ID")
    );
    let _ = ciborium::into_writer::<StreamItem, &mut dyn Write>(
        &StreamItem {
            part: "session".into(),
            proto: "raw".into(),
            id: None,
            version: None,
            dev: None,
            stream: None,
        },
        writer,
    );
    let _ = writer.flush();
}

fn handle_batch(item: StreamItem, writer: &mut dyn Write) {
    let id = match item.id {
        Some(id) => id,
        None => {
            warn!("no id was present for batch");
            return;
        }
    };
    info!("Received batch {}", id);
    let _ = ciborium::into_writer::<StreamItem, &mut dyn Write>(
        &StreamItem {
            id: Some(id),
            proto: "raw".into(),
            part: "batch".into(),
            dev: None,
            stream: None,
            version: None,
        },
        writer,
    );
    let _ = writer.flush();
    
    let file = std::fs::File::create(format!("/root/{:08x}.cbor", id)).unwrap();
    let _ = ciborium::into_writer(&item, &file);

    let datastream = match item.stream {
        Some(stream) => stream,
        None => {
            warn!("no stream in batch");
            return;
        }
    };
    let mut reader = BufReader::new(datastream.as_slice());
    loop {
        let item: BatchItem = match ciborium::from_reader(&mut reader) {
            Ok(item) => item,
            Err(ciborium::de::Error::Io(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(error) => {
                warn!("Failed to read batch item: {:?}", error);
                break;
            }
        };
        let seq = item.seq;
        let item: BatchItemData = match ciborium::from_reader(item.data.as_slice()) {
            Ok(item) => item,
            Err(_) => {
                match ciborium::from_reader::<ciborium::Value, &[u8]>(item.data.as_slice()) {
                    Ok(item) => {
                        warn!("Failed to read batch item data, generic value: {:?}", item);
                        continue;
                    },
                    Err(error) => {
                        warn!(
                            "Failed to read batch item data. Data was {:?}. Error was {:?}",
                            hex::encode(item.data), error
                        );
                        continue;
                    }
                };
            }
        };
        trace!("Batch item {} datum: {:?}", seq, item);
    }
}

fn handle_data_stream(stream: TcpStream) {
    info!("Incoming TCP connection");
    let _ = stream.set_read_timeout(Some(Duration::new(60, 0)));

    let mut writer = BufWriter::new(&stream);
    let mut reader = BufReader::new(&stream);

    loop {
        let item: StreamItem = ciborium::from_reader(&mut reader).unwrap();
        match item.part.as_str() {
            "session" => handle_session(item, &mut writer),
            "batch" => handle_batch(item, &mut writer),
            _ => {
                warn!("Unrecognized part {:?}", item.part);
                continue;
            }
        }
    }
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
        let listener = TcpListener::bind("0.0.0.0:1337").unwrap();
        for stream in listener.incoming() {
            let stream = match stream {
                Err(_) => continue,
                Ok(stream) => stream,
            };
            thread::spawn(move || {
                handle_data_stream(stream);
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
