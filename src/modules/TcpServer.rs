use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

//------------------------------------------------------------------------------
pub trait TcpLifecyle {
    fn on_connect(&mut self, addr: SocketAddr);
    fn on_receive(&mut self, buf: &[u8; 1024], bytes: &usize, addr: SocketAddr);
    fn on_disconect(&mut self, addr: SocketAddr);
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
        T: TcpLifecyle + Send + 'static
    {
        #[allow(clippy::never_loop)]
        loop {
            let (mut socket, addr) = self.listener.accept().await?;
            let arc_ref = Arc::clone(&prog);

            tokio::spawn(async move {
                {
                    let mut prog_ref = arc_ref.lock().await;
                    (*prog_ref).on_connect(addr.clone());
                }
                
                let mut buf = [0; 1024];
                
                loop {
                    let bytes_result = socket
                        .read(&mut buf)
                        .await;

                    let bytes = match bytes_result {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::ConnectionReset {
                                let mut prog_ref = arc_ref.lock().await;
                                (*prog_ref).on_disconect(addr.clone());
                            } else {
                                panic!("failed to read from socket; err = {:?}", e)
                            } 
                            return ;
                        }
                    };

                    {
                        let mut prog_ref = arc_ref.lock().await;
                        (*prog_ref).on_receive(&buf, &bytes, addr.clone());
                    }
                }
            });
        }
    }
}