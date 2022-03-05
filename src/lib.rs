#![deny(missing_debug_implementations)]

mod base_url;
mod client;
mod error;
mod inbound;

pub use client::{Config, EmailBody, ErrorReceipt, OutboundEmail, PostmarkClient, SendReceipt};
pub use error::{Error, PostmarkError};
pub use inbound::InboundEmail;
