mod modules;

use dotenv::{from_filename};
use serde_json::{Value, json};
// use tokio::{sync::Mutex};
use std::{
    env, 
    error::Error, 
    collections::HashMap, 
    net::{SocketAddr, TcpStream, ToSocketAddrs}, 
    sync::{Arc, Mutex}, 
    io::{Write, Read}
};

use modules::tcp_server::{TcpServer, TcpLifecyle, TcpStreamRef};

struct Peer {
    name: String,
    socket: TcpStreamRef
}

impl Peer {
    fn new(name: String, socket: TcpStreamRef) -> Self {
        Peer { name, socket }
    }
}

#[derive(Debug)]
enum ChatEvent {
    Register(String),
    Message(String),
}

struct Program {
    user_map: HashMap<SocketAddr, Peer>
}

impl Program {
    fn new() -> Self {
        Program { 
            user_map: HashMap::new()
        }
    }

    fn write_all_exclusevily(&self, addr: &SocketAddr, message: &str) {
        for (key, val) in self.user_map.iter() {
            if key.to_string() != addr.to_string() {
                let mut locked = match val.socket.lock() {
                    Ok(locked) => locked,
                    Err(poisoned) => {
                        poisoned.into_inner()
                    }
                };
                
                locked.write(
                    send_message_json(
                        &self.user_map.get(addr).unwrap().name, 
                        message.to_string()
                    ).as_bytes()
                ).unwrap();
            }
        }
    }
}

impl TcpLifecyle for Program {
    fn on_connect(&mut self, _socket: TcpStreamRef, _addr: SocketAddr) {
        // println!("{:?}", addr);
    }

    fn on_receive(&mut self, buf: &[u8; 1024], bytes: &usize, socket: TcpStreamRef, addr: SocketAddr) {
        let slice = &buf[..*bytes];
        
        let val = serde_json::from_slice::<serde_json::Value>(slice).unwrap_or_else(|e| {
            panic!("Error {} in: {}", String::from_utf8_lossy(slice), e);
        });
        let obj = parse_message(val);

        match obj {
            ChatEvent::Register(user) => {
                println!("{} entrou!!!", user);
                self.user_map.insert(addr, Peer::new(user, socket));
            },
            ChatEvent::Message(message) => {
                let user = self.user_map.get(&addr).unwrap();
                println!("{}: {}", user.name, message);
                
                self.write_all_exclusevily(&addr, message.as_str());
            }
        }
    }

    fn on_disconect(&mut self, _socket: TcpStreamRef, addr: SocketAddr) {
        self.user_map.remove(&addr);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    from_filename("./config/.env").ok();

    let host = env::var("DB_HOST")?;
    let port = env::var("DB_PORT")?;
    let addr = format!("{}:{}", host, port);

    TcpServer::new(&addr).await?.run(
        Arc::new(Mutex::new(Program::new()))
    ).await?;

    Ok(())
}

fn parse_message(value: Value) -> ChatEvent {
    let event = value["event"].as_str().unwrap();

    match event {
        "register" => {
            let name = value["content"]["name"].as_str().unwrap();

            ChatEvent::Register(name.to_string())
        },
        "message" => {
            let message = value["content"]["message"].as_str().unwrap();

            ChatEvent::Message(message.to_string())
        },
        _ => !panic!("Unknown event {}", event)
    }
}

fn send_message_json(name: &String, message: String) -> String {
    let json = json!({
        "event": "message",
        "content": {
            "name": name,
            "message": message
        }
    });

    json.to_string()
}