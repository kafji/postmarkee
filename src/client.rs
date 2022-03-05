/*!
HTTP client for Postmark's email API.

https://postmarkapp.com/developer/api/email-api
*/

use crate::{
    base_url::BaseUrl,
    error::{self, Error},
};
use chrono::{DateTime, Utc};
use http::{HeaderMap, HeaderValue, StatusCode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;

type HtmlBody<'a> = &'a str;

type TextBody<'a> = &'a str;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EmailBody<'a> {
    Html(HtmlBody<'a>),
    Text(TextBody<'a>),
    Both {
        html: HtmlBody<'a>,
        text: HtmlBody<'a>,
    },
}

impl<'a> EmailBody<'a> {
    /// Converts EmailBody sumtype into a pair of HTML body and text body.
    fn into_tuple(self) -> (Option<HtmlBody<'a>>, Option<TextBody<'a>>) {
        match self {
            EmailBody::Html(html) => (html.into(), None),
            EmailBody::Text(text) => (None, text.into()),
            EmailBody::Both { html, text } => (html.into(), text.into()),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Config {
    /// [Postmark's documentation](https://postmarkapp.com/developer/api/overview#endpoint-url).
    pub base_url: Option<BaseUrl>,

    /// [Postmark's documentation](https://postmarkapp.com/developer/api/overview#authentication).
    pub server_token: String,
}

#[derive(Debug)]
pub struct PostmarkClient {
    base_url: BaseUrl,
    http: Client,
}

impl PostmarkClient {
    pub fn new(
        Config {
            base_url,
            server_token,
        }: Config,
    ) -> Result<Self, Error> {
        let base_url = base_url.unwrap_or_default();

        let http = {
            let headers = {
                let mut m = HeaderMap::new();
                m.insert("content-type", HeaderValue::from_static("application/json"));
                m.insert("accept", HeaderValue::from_static("application/json"));
                m.insert("X-Postmark-Server-Token", {
                    let mut v = HeaderValue::from_str(&server_token).unwrap();
                    v.set_sensitive(true);
                    v
                });
                m
            };
            Client::builder().default_headers(headers).build().unwrap()
        };

        Ok(Self { base_url, http })
    }

    /// [Postmark's documentation](https://postmarkapp.com/developer/api/email-api#send-a-single-email).
    pub async fn send_email(
        &self,
        sender: &str,
        message_stream: Option<&str>,
        email: OutboundEmail<'_>,
    ) -> Result<SendReceipt, Error> {
        let url = {
            let mut url = self.base_url.clone().into_inner();
            url.path_segments_mut().unwrap().push("email");
            url
        };

        let recipients = email.recipients.join(",");
        let (html_body, text_body) = email.body.into_tuple();
        let payload = SendEmailPayload {
            from: sender,
            to: &recipients,
            subject: &email.subject,
            html_body,
            text_body,
            message_stream,
        };

        let request = self.http.post(url).json(&payload);

        let response = request.send().await?;

        let status_code = response.status();
        match status_code {
            StatusCode::OK => Ok(response.json().await?),
            StatusCode::UNPROCESSABLE_ENTITY => {
                let error = response.json().await?;
                Err(error::PostmarkError::UnprocessableEntity(error).into())
            }
            _ => Err(error::PostmarkError::Other(status_code).into()),
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct OutboundEmail<'a> {
    pub recipients: &'a [&'a str],
    pub subject: &'a str,
    pub body: EmailBody<'a>,
}

#[derive(Serialize, PartialEq, Copy, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct SendEmailPayload<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: Option<&'a str>,
    text_body: Option<&'a str>,
    message_stream: Option<&'a str>,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SendReceipt {
    pub to: String,
    pub submitted_at: DateTime<Utc>,

    #[serde(rename = "MessageID")]
    pub message_id: String,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorReceipt {
    pub error_code: u16,
    pub message: String,
}

impl fmt::Display for ErrorReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.error_code, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    #[test]
    fn deserialize_error_receipt() {
        let json = r#"
            {
                "ErrorCode": 405,
                "Message": "details"
            }
        "#;

        let receipt: ErrorReceipt = serde_json::from_str(json).unwrap();

        assert_eq!(
            receipt,
            ErrorReceipt {
                error_code: 405,
                message: "details".to_owned()
            }
        );
    }

    #[test]
    fn deserialize_send_receipt() {
        let json = r#"
            {
                "To": "receiver@example.com",
                "SubmittedAt": "2014-02-17T07:25:01.4178645-05:00",
                "MessageID": "0a129aee-e1cd-480d-b08d-4f48548ff48d",
                "ErrorCode": 0,
                "Message": "OK"
            }
        "#;

        let receipt: SendReceipt = serde_json::from_str(json).unwrap();

        assert_eq!(
            receipt,
            SendReceipt {
                to: "receiver@example.com".to_owned(),
                submitted_at: FixedOffset::west(5 * 60 * 60)
                    .ymd(2014, 2, 17)
                    .and_hms_nano(7, 25, 1, 417864500)
                    .with_timezone(&Utc),
                message_id: "0a129aee-e1cd-480d-b08d-4f48548ff48d".to_owned(),
            }
        );
    }
}
