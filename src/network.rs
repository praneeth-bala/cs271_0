use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::thread;
use serde::{Serialize, Deserialize};

use crate::blockchain::Block;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Request { client_id: u64, lamport_clock: u64 },
    Reply { client_id: u64, lamport_clock: u64 },
    Release { client_id: u64, lamport_clock: u64, block: Block },
}

pub struct Network {
    pub peers: HashMap<u64, TcpStream>,
    pub sender: Sender<Message>,
}

impl Network {
    pub fn new(sender: Sender<Message>) -> Self {
        Self {
            peers: HashMap::new(),
            sender,
        }
    }

    pub fn connect_to_peer(&mut self, client_id: u64, port: u16) {
        let address = format!("127.0.0.1:{}", port);
        match TcpStream::connect(&address) {
            Ok(stream) => {
                println!("Connected to peer at {}", address);
                let stream_clone = stream.try_clone().expect("Failed to clone TcpStream");
                let sender_clone = self.sender.clone();
                thread::spawn(move || {
                    handle_connection(stream_clone, sender_clone);
                });
                self.peers.insert(client_id, stream);
            }
            Err(e) => {
                println!("Failed to connect to {}: {}", address, e);
            }
        }
    }

    pub fn listen_for_peer(&mut self, client_id: u64, port: u16) {
        let address = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&address).unwrap();
        println!("Listening on {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Connected to peer at {}", address);
                    let stream_clone = stream.try_clone().expect("Failed to clone TcpStream");
                    let sender_clone = self.sender.clone();
                    thread::spawn(move || {
                        handle_connection(stream_clone, sender_clone);
                    });
                    self.peers.insert(client_id, stream);
                }
                Err(e) => {
                    println!("Connection failed: {}", e);
                }
            }
            break;
        }
        drop(listener);
    }

    pub fn send_message(&mut self, client_id: u64, message: Message) {
        if let Some(stream) = self.peers.get_mut(&client_id) {
            let serialized_message = serde_json::to_string(&message).unwrap();
            stream.write(serialized_message.as_bytes()).unwrap();
            stream.flush().unwrap();
            println!("Sent message to {}", client_id)
        } else {
            println!("No peer found with id: {}", client_id);
        }
    }
}

fn handle_connection(mut stream: TcpStream, sender: Sender<Message>) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by peer");
                break;
            }
            Ok(bytes_read) => {
                let message: Message = serde_json::from_slice(&buffer[..bytes_read]).unwrap();
                sender.send(message).expect("Failed to send message");
            }
            Err(e) => {
                println!("Failed to read from stream: {}", e);
                break;
            }
        }
    }
}
