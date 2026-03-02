use newsletter_service::configuration::get_configuration;
use newsletter_service::startup::Application;
use newsletter_service::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Create the subscriber
    let subscriber = get_subscriber("newletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Immediately panic if we cant read config
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;

    Ok(())
}
