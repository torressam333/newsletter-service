use std::net::TcpListener;

use newsletter_service::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Bubble up error if we failed to bind address
    let listener = TcpListener::bind("127.0.0.1:0")?;

    run(listener)?.await
}
