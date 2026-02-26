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
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();

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

    // Add some helpers to reduce code dupe...It's important to remain DRY

    // Random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    // Random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    // Random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// Get a test instance of `EmailClient`
    fn email_client(base_url: reqwest::Url, account_id: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            SecretString::new(Faker.fake::<String>().into()),
            account_id,                         // Moves the String in
            std::time::Duration::from_secs(10), // Senssible defailt for the tests suite
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let account_id: String = Faker.fake(); // Born here
        let base_url = reqwest::Url::parse(&mock_server.uri()).unwrap();

        // Pass a CLONE to the helper because we need the original
        // account_id later for the Mock path!
        let email_client = email_client(base_url, account_id.clone());

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path(format!("/api/send/{}", account_id)))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let account_id: String = Faker.fake(); // Born here
        let base_url = reqwest::Url::parse(&mock_server.uri()).unwrap();
        let email_client = email_client(base_url, account_id.clone());

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
        let account_id: String = Faker.fake(); // Born here
        let base_url = reqwest::Url::parse(&mock_server.uri()).unwrap();
        let email_client = email_client(base_url, account_id.clone());

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
        let account_id: String = Faker.fake(); // Born here
        let base_url = reqwest::Url::parse(&mock_server.uri()).unwrap();

        // Deviate from book - it just lets the suite hang for 10 seconds. I changed to use configuration in the struct instead
        let email_client = EmailClient::new(
            base_url,
            email(),
            SecretString::new(Faker.fake::<String>().into()),
            account_id,
            std::time::Duration::from_millis(50), // Client only waits 50ms
        );

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // 200ms delay, dont bottle neck the test suite
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(200));

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
