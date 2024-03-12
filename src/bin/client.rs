use tokio::net::TcpStream;

use learn_tokio_frame::{Connection, Frame};

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect() -> Client {
        let socket = TcpStream::connect("127.0.0.1:8080").await.unwrap();

        let connection = Connection::new(socket);

        Client { connection }
    }

    pub async fn addition(&mut self) -> learn_tokio_frame::Result<Frame> {
        let frame = Frame::Addition(10, 32);

        self.connection.write_frame(&frame).await;

        let response = self.connection.read_frame().await?;

        match response {
            Some(frame) => {
                println!("Server Response: {:#?}", &frame);
                Ok(frame)
            }
            None => {
                println!("Failed to get a response");
                Err("No response".into())
            }
        }
    }
}

#[tokio::main]
pub async fn main() {
    let mut c = Client::connect().await;
    c.addition().await;
}
