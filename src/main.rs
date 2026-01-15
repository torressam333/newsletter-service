use newsletter_service::configuration::get_configuration;
use newsletter_service::startup::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Immediately panic if we cant read config
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    // Bubble up error if we failed to bind address
    let listener = TcpListener::bind(address)?;

    run(listener)?.await
}
