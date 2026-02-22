use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

pub struct EmailClient {
    http_client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
    // This will be populated from APP_EMAIL_CLIENT__AUTHORIZATION_TOKEN
    pub authorization_token: SecretString,
}

#[derive(serde::Serialize)]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_content: String,
    text_content: String,
}

impl EmailClient {
    pub fn new(
        base_url: reqwest::Url,
        sender: SubscriberEmail,
        authorization_token: SecretString,
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/send/4390866", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: recipient.as_ref().to_owned(),
            subject: subject.to_owned(),
            html_content: html_content.to_owned(),
            text_content: text_content.to_owned(),
        };

        self.http_client
            .post(&url)
            // EXPOSE the secret only at the point of usage
            .header(
                "Authorization",
                format!("Bearer {}", self.authorization_token.expose_secret()),
            )
            .json(&request_body)
            .send() // <--- Don't forget to send!
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::SecretString;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let base_url =
            reqwest::Url::parse(&mock_server.uri()).expect("Failed to parse mock server tab URI");
        let secret_data: String = Faker.fake();

        // Into will convert String to expected Box<str> expected by SS::new()
        let email_client =
            EmailClient::new(base_url, sender, SecretString::new(secret_data.into()));

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1) // Should receive exactly 1 req set by the mock.
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await
            .expect("Failed to send email");
    }
}
