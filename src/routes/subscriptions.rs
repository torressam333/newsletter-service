use crate::domain::{NewSubscriber, SubscriberName};
use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
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
// Subscribe is the route handler called in startup.rs. The entry point for this endpoint.
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    // 0 gives us access to FormData coming from the web::Form wrapper
    let new_subscriber = NewSubscriber {
        email: form.0.email,
        name: SubscriberName::parse(form.0.name).expect("Name validation failed."),
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
} // Request span gets dropped here and span is exited

/// Returns true if input satisfies all validationo for name field
pub fn is_valid_name(name: &str) -> bool {
    let is_empty_or_whitespace = name.trim().is_empty();

    // Some chars are actually composed of multiple bytes
    let is_too_long = name.graphemes(true).count() > 256;

    // Iterate over all chars to check if any of them are in the forbidden chars array
    let forbidden_chars = ['/', '(', ')', ',', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_chars = name.chars().any(|g| forbidden_chars.contains(&g));

    // return false if any conditions have been violated
    !(is_empty_or_whitespace || is_too_long || contains_forbidden_chars)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.as_ref(),
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
