use std::error::Error;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

//------------------------------------------------------------------------------
pub trait TcpLifecyle {
    fn on_receive(&mut self, buf: &[u8; 1024], bytes: &usize, who: String);
    fn on_disconect(&mut self, who: String);
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
    
    pub async fn run<T>(&self, prog: &mut T) -> Result<(), Box<dyn Error>> 
    where 
        T: TcpLifecyle + Send
    {
        #[allow(clippy::never_loop)]
        loop {
            let (mut socket, _) = self.listener.accept().await?;
            
            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    let mut buf = [0; 1024];
                    
                    loop {
                        let who = socket.peer_addr().ok().unwrap().to_string();
                        let n = socket
                            .read(&mut buf)
                            .await
                            .unwrap_or_else(|e| {
                                if e.kind() == std::io::ErrorKind::ConnectionReset {
                                    prog.on_disconect(who.clone());
                                }
                                panic!("failed to read from socket; err = {:?}", e)
                            });

                        prog.on_receive(&buf, &n, who.clone());
                        if n == 0 { return; }
                        // socket
                        //     .write_all(&buf[0..n])
                        //     .await
                        //     .expect("failed to write data to socket");
                    }
                });
            });
        }
    }
}