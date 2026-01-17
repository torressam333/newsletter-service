use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgConnection;
use std::net::TcpListener;

pub fn run(listener: TcpListener, connection: PgConnection) -> Result<Server, std::io::Error> {
    // Wrap the DB connection in a smart pointer
    let connection = web::Data::new(connection);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Get a pointer copy and attach to the app's state
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
