use newsletter_service::configuration::get_configuration;
use newsletter_service::email_client::EmailClient;
use newsletter_service::startup::run;
use newsletter_service::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;
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
        PgPoolOptions::new().connect_lazy_with(configuration.database.connection_options());

    // Build the email client using config
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let formatted_url = reqwest::Url::parse(&configuration.email_client.base_url)
        .expect("Failed to parse base url");

    let timeout = configuration.email_client.timeout();

    let email_client = EmailClient::new(
        formatted_url,
        sender_email,
        configuration.email_client.authorization_token,
        configuration.email_client.mailtrap_account_id,
        timeout,
    );

    // Configure server address
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    // Bubble up error if we failed to bind address
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool, email_client)?.await
}
