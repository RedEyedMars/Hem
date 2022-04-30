#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket_contrib::serve::StaticFiles;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::{BufReader, Read, Write};

extern crate fern;
#[macro_use]
extern crate log;
extern crate uuid;
use uuid::Uuid;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Level {
    filename: String,
    body: String,
}

use rocket::response::NamedFile;
#[get("/<filename2>")]
async fn load_img(filename2: u16) -> NamedFile {
    let mut filename = String::from("./res/img/static/");

    use maplit::hashmap;
    let namemap = hashmap! {
        1u16 => "grass",
        2u16 => "wall",
        16u16 => "player",
        32u16 => "monster",
        128u16 => "particle",
    };
    filename.push_str(namemap[&(filename2 / 256u16)]);
    filename.push_str("_");
    filename.push_str((filename2 % 256).to_string().as_str());
    filename.push_str(".png");

    NamedFile::open(filename.as_str())
        .await
        .expect("Could not open img")
}

#[get("/<filename2>")]
async fn load_icon(filename2: String) -> NamedFile {
    let mut filename = String::from("./res/img/static/");
    filename.push_str(filename2.as_str());

    NamedFile::open(filename.as_str())
        .await
        .expect("Could not open icon")
}

#[get("/res/levels/<filename2>")]
fn load_level(filename2: String) -> String {
    let mut filename = String::from("./res/levels/");
    filename.push_str(filename2.as_str());
    match File::open(filename.clone()) {
        Ok(file) => {
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();

            if let Ok(_) = buf_reader.read_to_string(&mut contents) {
                contents
            } else {
                String::from("Failed to read file")
            }
        }
        Err(err) => {
            info!("{}", err);
            String::from("Failed to find file")
        }
    }
}

#[post("/", format = "json", data = "<level>")]
fn save_level(level: Json<Level>) {
    info!("{}, {}", level.filename.clone(), level.body.clone());
    let mut file = File::create(level.filename.clone()).expect("Could not create file");
    file.write_all(level.body.clone().into_bytes().as_slice())
        .expect("Could not save level");
}

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
#[get("/<file..>")]
async fn load_wasm(file: PathBuf) -> NamedFile {
    NamedFile::open(Path::new("./dist/").join(file))
        .await
        .expect("Could not open wasm")
}

use rocket::{data::ByteUnit, Data, State};

struct Client();

use std::sync::{Arc, Mutex};
use std::thread;

use websocket::sync::Server;
use websocket::sync::Writer;

use std::sync::mpsc::channel;
use websocket::OwnedMessage;

use std::collections::HashMap;

enum SocketMessage {
    Close(Uuid),
    Ping(OwnedMessage),
    NewClient,
    Binary(usize, Vec<u8>),
}

#[rocket::main]
async fn main() -> (Result<(), String>) {
    setup_logger().expect("logger setup failed");

    thread::spawn(|| {
        info!("Bind 13125");
        let server = Server::bind("127.0.0.1:13125").unwrap();

        //let mut peers: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::with_capacity(256)));
        let mut senders = Arc::new(Mutex::new(HashMap::new()));
        let mut id_map: Arc<Mutex<HashMap<usize, Uuid>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut ids: Arc<Mutex<Vec<Uuid>>> = Arc::new(Mutex::new(Vec::new()));
        for request in server.filter_map(Result::ok) {
            // Spawn a new thread for each connection.
            //let peers_clone = Arc::clone(&peers);
            let mut sc = Arc::clone(&senders);
            let mut imc = Arc::clone(&id_map);
            let mut ic = Arc::clone(&ids);
            let mut new_index = 0usize;
            {
                new_index = ic.lock().unwrap().len();
            }
            let new_id = Uuid::new_v4();
            thread::spawn(move || {
                if !request.protocols().contains(&"rust-websocket".to_string()) {
                    info!("Reject non-rust-websocket");
                    request.reject().unwrap();
                    return;
                }

                let mut client = request.use_protocol("rust-websocket").accept().unwrap();

                let ip = client.peer_addr().unwrap();
                info!("Connection from {}", ip);
                let (mut receiver, mut sender) = client.split().unwrap();
                let (mut tx, mut rx) = channel();
                thread::spawn(move || {
                    for message in receiver.incoming_messages() {
                        let message = message.unwrap();

                        info!("------------------------Received: {:?}", &message);
                        match message {
                            OwnedMessage::Close(_) => {
                                tx.send(SocketMessage::Close(new_id)).unwrap();
                                info!("Client {} disconnected", ip);
                                return;
                            }
                            OwnedMessage::Ping(ping) => {
                                tx.send(SocketMessage::Ping(OwnedMessage::Pong(ping)))
                                    .unwrap();
                            }
                            OwnedMessage::Binary(input) if *input.get(0).unwrap() == 0u8 => {
                                tx.send(SocketMessage::NewClient).unwrap();
                            }
                            OwnedMessage::Binary(input) if *input.get(0).unwrap() >= 1u8 => {
                                tx.send(SocketMessage::Binary(
                                    input.get(2).unwrap().clone() as usize,
                                    input,
                                ))
                                .unwrap();
                            }
                            _ => {} //sender.send_message(&message).unwrap(),
                        }
                    }
                });

                sc.lock().unwrap().insert(new_id, sender);
                imc.lock().unwrap().insert(new_index, new_id);
                ic.lock().unwrap().push(new_id);
                while let Ok(msg) = rx.recv() {
                    match msg {
                        SocketMessage::Close(close_id) => {
                            {
                                let mut ic_lock = ic.lock().unwrap();
                                let mut sc_lock = sc.lock().unwrap();
                                let mut imc_lock = imc.lock().unwrap();
                                let close_index =
                                    ic_lock.iter().position(|x| *x == close_id).unwrap();
                                for (i, sender_id) in ic_lock.iter_mut().enumerate() {
                                    if i < close_index {
                                        sc_lock
                                            .get_mut(sender_id)
                                            .unwrap()
                                            .send_message(&OwnedMessage::Binary(vec![
                                                255,
                                                close_index as u8,
                                                i as u8,
                                            ]))
                                            .unwrap();
                                    } else if i > close_index {
                                        sc_lock
                                            .get_mut(sender_id)
                                            .unwrap()
                                            .send_message(&OwnedMessage::Binary(vec![
                                                255,
                                                close_index as u8,
                                                (i as u8) - 1,
                                            ]))
                                            .unwrap();
                                        let previous_id = imc_lock.remove(&i).unwrap().clone();
                                        imc_lock.insert(i - 1, previous_id);
                                    }
                                }
                                ic_lock.remove(close_index);
                                sc_lock.remove(&close_id);
                            }
                            info!("Closed");
                            return;
                        }
                        SocketMessage::Ping(pong) => sc
                            .lock()
                            .unwrap()
                            .get_mut(&new_id)
                            .unwrap()
                            .send_message(&pong)
                            .unwrap(),
                        SocketMessage::NewClient => {
                            info!("New client {}", new_id);
                            let ic = vec![0]
                                .iter()
                                .map(|x| *x)
                                .chain(
                                    ic.lock()
                                        .unwrap()
                                        .iter()
                                        .enumerate()
                                        .map(|(x, _)| x as u8)
                                        .rev(),
                                )
                                .collect();
                            sc.lock()
                                .unwrap()
                                .get_mut(&new_id)
                                .unwrap()
                                .send_message(&OwnedMessage::Binary(ic))
                                .unwrap();
                        }
                        SocketMessage::Binary(dest, input) => {
                            info!("dest: {}", dest);
                            sc.lock()
                                .unwrap()
                                .get_mut(&(imc.lock().unwrap().get(&(dest)).unwrap()))
                                .unwrap()
                                .send_message(&OwnedMessage::Binary(input))
                                .unwrap();
                        }
                    };
                }
            });
        }
    });

    let result = rocket::ignite()
        .mount("/", StaticFiles::from("./dist"))
        .mount("/ent/pkg", routes![load_wasm])
        .mount("/l", routes![load_level, save_level])
        .mount("/i", routes![load_img])
        .mount("/b", routes![load_icon])
        .launch()
        .await;
    result.map_err(|e| e.to_string())
}
