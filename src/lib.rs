pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

// Put this anywhere in src/lib.rs
async fn _dummy_query(pool: sqlx::PgPool) {
    sqlx::query!("SELECT id FROM subscriptions")
        .fetch_one(&pool)
        .await
        .ok();
}
