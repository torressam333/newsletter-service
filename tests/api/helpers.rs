use newsletter_service::configuration::{DatabaseSettings, get_configuration};
use newsletter_service::startup::Application;
use newsletter_service::startup::get_connection_pool;
use newsletter_service::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgConnection;
use sqlx::{Connection, Executor, PgPool};
use std::sync::LazyLock;
use uuid::Uuid;

// Ensure tracing stack is only initialized once via LazyLock
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);

        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let configuration = {
        let mut con = get_configuration().expect("Failed to read config.");
        con.database.database_name = Uuid::new_v4().to_string();
        con.application.port = 0;
        con
    };

    // 1. PROVISION: This MUST happen first.
    // It creates the DB and runs migrations so the tables actually exist.
    configure_database(&configuration.database).await;

    // 2. BUILD APP: This calls get_connection_pool internally (via Application::build)
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application instance");

    let address = format!("http://127.0.0.1:{}", application.port());

    tokio::spawn(application.run_until_stopped());

    // 3. TEST POOL: Now it's safe to call get_connection_pool for the test logic
    // because step #1 guaranteed the database and tables are ready.
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // 1. SECURITY FIX: Validate that the database name is a valid UUID
    // This prevents SQL injection even though we use format!() below.
    uuid::Uuid::parse_str(&config.database_name)
        .expect("Security Alert: Invalid database name provided.");

    // Create the db using Postgres's default "maintenance db"
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: "password".to_string().into(),
        ..config.clone()
    };

    let mut connection = PgConnection::connect_with(
        // 1. Call the method (returns SecretString)
        // 2. Expose it (returns &str)
        &maintenance_settings.connection_options(),
    )
    .await
    .expect("Failed to connect to Postgres");

    // Use the superuser to create the DB and ASSIGN ownership to the app user
    // and double quotes around the identifier for extra safety
    connection
        .execute(
            format!(
                r#"CREATE DATABASE "{}" OWNER {};"#,
                config.database_name, config.username
            )
            .as_str(),
        )
        .await
        .expect("Failed to create database");

    // Migrate the db
    let connection_pool = PgPool::connect_with(
        // Uncloak the secret at the boundary
        config.connection_options(),
    )
    .await
    .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
