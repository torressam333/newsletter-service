use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    /* DEV NOTE: PII & Compliance (GDPR/CCPA)
    We are logging email and name here for debugging purposes.
    In a high compliance production environment, this should be avoided
    or masked to prevent PII leakage into log aggregation systems (e.g. Dynatrace or Datadog).
    Logs should also be subject to a TTL (Time-To-Live) policy to honor
    'Right to be Forgotten' requests.
    */
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Adding new subscriber.", %request_id, subscriber_email = %form.email, subscriber_name = %form.name);

    // We use .instrument() or manually enter/exit, but for now, let's look at the logs:
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    // Use get_ref to gen an immutable ref to the Pg pool wrapped in web::Data
    .execute(pool.get_ref())
    // First attach instrumentation then await it
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            // Recording the error as a structured field - for now it will fall outside of query_span
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
} // Request span gets dropped here and span is exited
