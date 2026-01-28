use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

/* DEV NOTE: PII & Compliance (GDPR/CCPA)
    We are logging email and name here for debugging purposes.
    In a high compliance production environment, this should be avoided
    or masked to prevent PII leakage into log aggregation systems (e.g. Dynatrace or Datadog).
    Logs should also be subject to a TTL (Time-To-Live) policy to honor
    'Right to be Forgotten' requests.
*/
#[tracing::instrument(
    name="Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
} // Request span gets dropped here and span is exited

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);

        // Easy to miss a single letter return ...commenting :)
        e

        // Use ? to return early if fn fails...will propagate for now and properly handle later
    })?;

    Ok(())
}
