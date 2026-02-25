use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

pub struct EmailClient {
    http_client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
    // This will be populated from APP_EMAIL_CLIENT__AUTHORIZATION_TOKEN
    pub authorization_token: SecretString,
    mailtrap_account_id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_content: &'a str,
    text_content: &'a str,
}

impl EmailClient {
    pub fn new(
        base_url: reqwest::Url,
        sender: SubscriberEmail,
        authorization_token: SecretString,
        mailtrap_account_id: String,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
            mailtrap_account_id,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let path = format!("api/send/{}", self.mailtrap_account_id);

        let url = self
            .base_url
            .join(&path)
            .expect("Failed to join base URL with dynamic Mailtrap path");

        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_content,
            text_content,
        };

        self.http_client
            .post(url)
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
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::SecretString;
    use wiremock::Request;
    use wiremock::matchers::any;
    use wiremock::matchers::{header, header_exists, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Custom impl for checking body of req during tests
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse body as JSON value
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                dbg!(&body);
                // Check that all mandatory fields ar populated w/o inspecting
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlContent").is_some()
                    && body.get("TextContent").is_some()
            } else {
                // Req doesnt match, failed parsing
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let base_url =
            reqwest::Url::parse(&mock_server.uri()).expect("Failed to parse mock server tab URI");
        let secret_data: String = Faker.fake();
        let account_id: String = Faker.fake();

        // Into will convert String to expected Box<str> expected by SS::new()
        let email_client = EmailClient::new(
            base_url,
            sender,
            SecretString::new(secret_data.into()),
            account_id.clone(),
        );

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path(format!("/api/send/{}", account_id)))
            .and(SendEmailBodyMatcher)
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

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let account_id: String = Faker.fake();
        let secret_data: String = Faker.fake();
        let base_url =
            reqwest::Url::parse(&mock_server.uri()).expect("Failed to parse mock server tab URI");

        let email_client = EmailClient::new(
            base_url,
            sender,
            SecretString::new(secret_data.into()),
            account_id.clone(),
        );

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let account_id: String = Faker.fake();
        let secret_data: String = Faker.fake();
        let base_url =
            reqwest::Url::parse(&mock_server.uri()).expect("Failed to parse mock server tab URI");

        let email_client = EmailClient::new(
            base_url,
            sender,
            SecretString::new(secret_data.into()),
            account_id.clone(),
        );

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let account_id: String = Faker.fake();
        let secret_data: String = Faker.fake();
        let base_url =
            reqwest::Url::parse(&mock_server.uri()).expect("Failed to parse mock server tab URI");

        let email_client = EmailClient::new(
            base_url,
            sender,
            SecretString::new(secret_data.into()),
            account_id.clone(),
        );

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // 3 min delay, that's forever in internet time lol
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        assert_err!(outcome);
    }
}
