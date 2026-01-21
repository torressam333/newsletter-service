use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
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
    log::info!(
        "Request ID {} - Adding '{}' '{}' as a new subscriber.",
        request_id,
        form.email,
        form.name
    );
    log::info!(
        "Request ID {} - Saving new subscriber to the database.",
        request_id
    );
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
    .await
    {
        Ok(_) => {
            log::info!(
                "Request ID {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!(
                "Request ID {} - Failed to execute query: {:?}",
                e,
                request_id
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
