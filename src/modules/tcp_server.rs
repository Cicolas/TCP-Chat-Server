use std::cell::{RefCell, Ref};
use std::error::Error;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write, BufRead};
use std::net::TcpStream;
use std::net::{TcpListener};
use std::thread;
use std::time::Duration;

// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::sync::{Mutex};

//------------------------------------------------------------------------------
pub type TcpStreamRef = Arc<Mutex<TcpStream>>;
pub trait TcpLifecyle {
    // type a: String;
    fn on_connect(&mut self, socket: TcpStreamRef, addr: SocketAddr);
    fn on_receive(&mut self, buf: &[u8; 1024], bytes: &usize, socket: TcpStreamRef, addr: SocketAddr);
        // -> (&mut broadcast::Sender<String>, &mut broadcast::Receiver<String>);
        // -> Pin<Box<dyn Future<Output = ()>>>;
    fn on_disconect(&mut self, socket: TcpStreamRef, addr: SocketAddr);
}

//------------------------------------------------------------------------------

pub struct TcpServer {
    listener: TcpListener
}

impl TcpServer {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind(addr)?;
        println!("Listening on: {}", addr);

        Ok(TcpServer {
            listener: listener
        })
    }
    
    pub async fn run<T>(&self, prog: Arc<Mutex<T>>) -> Result<(), Box<dyn Error>> 
    where
        T: TcpLifecyle + Send + 'static
    {
        #[allow(clippy::never_loop)]
        loop {
            let (socket, addr) = self.listener.accept()?;
            let socket_ref: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(socket)); 
            let arc_ref = Arc::clone(&prog);

            thread::spawn(move || {
                {
                    let locked = socket_ref.lock().unwrap();
                    locked.set_read_timeout(Some(Duration::from_millis(3000))).unwrap();
                    locked.set_write_timeout(Some(Duration::from_millis(3000))).unwrap();
                }
                
                {
                    let mut prog_ref = arc_ref.lock().unwrap();
                    (prog_ref).on_connect(socket_ref.clone(), addr.clone());
                }

                let mut buf = [0; 1024];

                loop {
                    let bytes;

                    {
                        bytes = socket_ref.lock().unwrap().read(&mut buf);
                    }

                    
                    if bytes.is_err() {
                        let e = bytes.err().unwrap();
                        if e.kind() == std::io::ErrorKind::ConnectionReset {
                            let mut prog_ref = arc_ref.lock().unwrap();
                            (prog_ref).on_disconect(socket_ref.clone(), addr.clone());
                        } 
                        // else if e.kind() == std::io::ErrorKind::TimedOut {
                        //     // println!("Timed Out: {}", addr);
                        //     continue;
                        // }
                        panic!("failed to read from socket; err = {:?}", e);
                    }
                    
                    let n = bytes.unwrap();

                    {
                        let mut prog_ref = arc_ref.lock().unwrap();
                        (prog_ref).on_receive(&buf, &n, socket_ref.clone(), addr.clone());
                    }
                }
            });
        }
    }
}