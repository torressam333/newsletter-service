use newsletter_service::configuration::get_configuration;
use newsletter_service::startup::run;
use newsletter_service::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Create the subscriber
    let subscriber = get_subscriber("newletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Immediately panic if we cant read config
    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool =
        PgPool::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    // Bubble up error if we failed to bind address
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await
}
