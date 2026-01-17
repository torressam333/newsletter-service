use newsletter_service::configuration::get_configuration;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

/*
[BOOK DIVERGENCE]

I'm diverging from the book's initial implementation here.
I make my test setup async because initializing the database connection is an async operation.
While it adds a tiny bit of boilerplate to each test call, it keeps the setup logic
honest—I am performing I/O before the test starts, so the function signature should reflect that
*/
async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");
    let server =
        newsletter_service::startup::run(listener, connection).expect("Failed to bind address");

    tokio::spawn(server);
    // Give the server a moment to start listening
    std::thread::sleep(std::time::Duration::from_millis(50));

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn subuscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app().await;
    let configuration = get_configuration().expect("Failed to get configuration.");
    let connection_string = configuration.database.connection_string();

    // We have to manually bring Connection trait into scope for pg connection to work
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres DB");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch savved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("emaik=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            // Add customized message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
