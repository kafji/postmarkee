/*!
Types for Postmark's inbound webhook.

https://postmarkapp.com/developer/webhooks/inbound-webhook
*/

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use quickcheck_derive::Arbitrary;

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
#[serde(rename_all = "PascalCase")]
pub struct InboundEmail {
    pub from_name: String,
    pub message_stream: String,
    pub from_full: Participant,
    pub to_full: Vec<Participant>,
    pub cc_full: Vec<Participant>,
    pub bcc_full: Vec<Participant>,
    pub original_recipient: String,
    pub subject: String,

    #[serde(rename = "MessageID")]
    pub message_id: String,

    pub reply_to: String,
    pub mailbox_hash: String,

    #[serde(with = "rfc2822_serde")]
    #[cfg_attr(test, arbitrary(generator = "gen_date_time_utc"))]
    pub date: DateTime<Utc>,

    pub text_body: String,
    pub html_body: String,
    pub stripped_text_reply: String,
    pub tag: String,
    pub headers: Vec<Header>,
    pub attachments: Vec<Attachment>,
}

#[cfg(test)]
fn gen_date_time_utc(g: &mut quickcheck::Gen) -> DateTime<Utc> {
    use chrono::TimeZone;
    use quickcheck::Arbitrary;
    Utc.timestamp(u32::arbitrary(g) as _, 0)
}

mod rfc2822_serde {
    use chrono::{DateTime, Utc};
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    const FORMAT: &str = "%a, %-d %b %Y %H:%M:%S %:z";

    pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = DateTime<Utc>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "RFC2822 string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                DateTime::parse_from_str(v, FORMAT)
                    .map(|x| x.with_timezone(&Utc))
                    .map_err(|x| de::Error::custom(x))
            }
        }
        d.deserialize_str(V)
    }

    pub fn serialize<S>(v: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str = v.format(FORMAT).to_string();
        s.serialize_str(&str)
    }
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
#[serde(rename_all = "PascalCase")]
pub struct Participant {
    pub email: String,
    pub name: String,
    pub mailbox_hash: String,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
#[serde(rename_all = "PascalCase")]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
#[serde(rename_all = "PascalCase")]
pub struct Attachment {
    pub name: String,

    #[serde(with = "base64_serde")]
    #[cfg_attr(test, arbitrary(generator = "gen_bytes"))]
    pub content: Bytes,

    pub content_type: String,
    pub content_length: i32,
}

#[cfg(test)]
fn gen_bytes(g: &mut quickcheck::Gen) -> Bytes {
    use quickcheck::Arbitrary;
    Bytes::from(Vec::<u8>::arbitrary(g))
}

mod base64_serde {
    use bytes::Bytes;
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn deserialize<'de, D>(d: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Bytes;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "Base64 string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                base64::decode(v)
                    .map(Bytes::from)
                    .map_err(|x| de::Error::custom(x))
            }
        }
        d.deserialize_str(V)
    }

    pub fn serialize<S>(v: &Bytes, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&base64::encode(&*v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn test_serde_is_identity(v: InboundEmail) -> bool {
        serde_json::from_str::<InboundEmail>(&serde_json::to_string(&v).unwrap()).unwrap() == v
    }
}
