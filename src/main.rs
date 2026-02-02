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

    // Only establish conn when pool is used for first time, not async anymore (lazy)
    let connection_pool =
        PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
            .expect("Failed to create Postgres connection pool");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    // Bubble up error if we failed to bind address
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await
}
