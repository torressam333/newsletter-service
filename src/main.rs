use newsletter_service::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    run().await
}
