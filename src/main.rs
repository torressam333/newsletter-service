use env_logger::Env;
use newsletter_service::configuration::get_configuration;
use newsletter_service::startup::run;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // init() calls set_logger
    // Fall back to printing inifo levl or above for all logs
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Immediately panic if we cant read config
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    // Bubble up error if we failed to bind address
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await
}
