use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use tokio::sync::{Mutex};

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
        let listener = TcpListener::bind(addr).await?;
        println!("Listening on: {}", addr);

        Ok(TcpServer {
            listener: listener
        })
    }
    
    pub async fn run<T>(&self, prog: Arc<Mutex<T>>) -> Result<(), Box<dyn Error>> 
    where
        T: TcpLifecyle + Send + Sync + 'static
    {
        #[allow(clippy::never_loop)]
        loop {
            let (mut socket, addr) = self.listener.accept().await?;
            let socket_ref = Arc::new(Mutex::new(socket));
            let arc_ref = Arc::clone(&prog);

            tokio::spawn(async move {
                {
                    let mut prog_ref = arc_ref.lock().await;
                    (*prog_ref).on_connect(socket_ref.clone(), addr.clone());
                }
                
                let mut buf = [0; 1024];

                loop {
                    let bytes = (socket_ref.lock().await)
                        .read(&mut buf)
                        .await;

                    if bytes.is_err() {
                        let e = bytes.err().unwrap();
                        if e.kind() == std::io::ErrorKind::ConnectionReset {
                            let mut prog_ref = arc_ref.lock().await;
                            (*prog_ref).on_disconect(socket_ref.clone(), addr.clone());
                        } 
                        panic!("failed to read from socket; err = {:?}", e);
                    }

                    let n = bytes.unwrap();

                    {
                        let mut prog_ref = arc_ref.lock().await;
                        (*prog_ref).on_receive(&buf, &n, socket_ref.clone(), addr.clone());
                    }
                
                    // socket.write_all(&buf[..n]).await.unwrap();
                    // socket.write_all(&buf[..bytes.unwrap()]).await.unwrap();
                }
            });
        }
    }
}