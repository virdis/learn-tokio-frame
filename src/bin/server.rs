use tokio::net::TcpListener;

#[tokio::main]
pub async fn main() -> learn_tokio_frame::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    learn_tokio_frame::server::run(listener).await;
    Ok(())
}
