mod modules;

use dotenv::{from_filename};
use serde_json::Value;
use std::{env, error::Error, collections::HashMap, clone};

use modules::TcpServer::{TcpServer, TcpLifecyle};

#[derive(Debug)]
enum ChatEvent {
    Register(String),
    Message(String),
}

struct Program {
    user_map: HashMap<String, String>
}

impl Program {
    fn new() -> Self {
        Program { 
            user_map: HashMap::new()
        }
    }
}

impl TcpLifecyle for Program {
    fn on_receive(&mut self, buf: &[u8; 1024], bytes: &usize, who: String) {
        let val = serde_json::from_slice::<serde_json::Value>(&buf[..*bytes]).unwrap();
        let obj = parse_message(val);

        // println!("{:?}", obj);

        match obj {
            ChatEvent::Register(user) => {
                println!("{} entrou!!!", user);
                self.user_map.insert(who, user);
            },
            ChatEvent::Message(message) => {
                let user = self.user_map.get(&who).unwrap();
                println!("{}: {}", user, message);
            },
            _ => {}
        }
    }

    fn on_disconect(&mut self, who: String) {
        self.user_map.remove(&who);
        // println!("{:?}", self.user_map);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    from_filename("./config/.env").ok();

    let host = env::var("DB_HOST")?;
    let port = env::var("DB_PORT")?;
    let addr = format!("{}:{}", host, port);

    let mut prog = Program::new();

    TcpServer::new(&addr).await?.run(&mut prog).await?;

    Ok(())
}

fn parse_message(value: Value) -> ChatEvent {
    let event = value["event"].as_str().unwrap();
    // println!("{:?}", event);

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