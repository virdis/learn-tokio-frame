use std::{result, sync::Arc, time::Duration};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::Semaphore,
    time,
};

use crate::Connection;

const MAX_CONNECTIONS: usize = 250;

// TODO: Add graceful shutdown logic
// Per connection handler
#[derive(Debug)]
struct Handler {
    connection: Connection,
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        let rframe = self.connection.read_frame().await;

        let oframe = match rframe {
            Ok(opt_frame) => opt_frame,
            Err(e) => {
                println!("Failed reading the frame error {}", e);
                None
            }
        };
        match oframe {
            Some(frame) => {
                let result = self.handle_frame(frame).await;
                result.or_else(|e| Err(e))
            }
            None => Ok(()),
        }
    }

    async fn handle_frame(&mut self, frame: crate::Frame) -> Result<(), crate::Error> {
        let op_result = match frame {
            crate::Frame::Addition(x, y) => x + y,
            crate::Frame::Subtraction(x, y) => x - y,
            crate::Frame::Multiplication(x, y) => x * y,
            crate::Frame::OpResult(r) => r,
        };
        let response = crate::Frame::OpResult(op_result);
        println!("Respone: {:#?}", &response);
        self.connection.write_frame(&response).await
    }
}

pub async fn run(listener: TcpListener) {
    let mut server = Listener {
        listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
    };

    server.run().await;
}

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
}

impl Listener {
    // TODO: add logging library
    async fn run(&mut self) -> crate::Result<()> {
        println!("Incoming connection!");

        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let socket = self.accept().await?;

            let mut handler = Handler {
                connection: Connection::new(socket),
            };

            tokio::spawn(async move {
                if let Err(error) = handler.run().await {
                    eprintln!("Connection error {:#?}", error);
                }

                drop(permit);
            });
        }
    }

    async fn accept(&mut self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;

            backoff *= 2;
        }
    }
}
