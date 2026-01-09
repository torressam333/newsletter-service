use newsletter_service::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Bubble up error if we failed to bind address
    run()?.await
}
