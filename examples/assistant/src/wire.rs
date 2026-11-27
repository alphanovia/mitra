//! The HTTP protocol between the clients and the server. The token goes
//! in `Authorization: Bearer <token>`; bodies are JSON. A call with no
//! token is 401; what an unknown token gets is the tied code's to say:
//!
//!   GET  /me                    -> 200 Me                 | 401 unknown token
//!   POST /messages/{n}  Post    -> 200 Received           | 403 refused (unknown token,
//!                                                           or the gateway says no)
//!                                                         | 400 not JSON | 408 too slow
//!                                                         | 413 too long
//!   GET  /messages?after=N      -> 200 [Reply], at most 50 (ask again after the last);
//!                                  [] for an unknown token
//!   GET  /messages/{n}/status   -> 200 Status             | 404 no such message
//!   GET  /typing                -> 200 Typing (false for an unknown token)
//!   POST /replies/{n}/ack       -> 204, whoever asks (it marks only what the token may)
//!
//! 503: the server is busy (a connection limit, or its queue). The call was
//! not taken, so it may be sent again (the gateway keeps a repeated id
//! once, `Gateway::retry_safe`). These shapes carry the model's values;
//! none of them is in the model.

use serde::{Deserialize, Serialize};

/// Who a token is, and where its device stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Me {
    pub household: i64,
    pub device: String,
    pub number: i64,
    /// The highest number this device has used.
    pub count: i64,
    /// The highest reply this device said it got.
    pub last_delivered: i64,
}

/// What a message carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Text,
    Voice,
}

/// A message posted; its number is in the path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
    pub kind: Kind,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Received {
    pub received: i64,
}

/// A reply of the assistant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reply {
    pub n: i64,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Received,
    Read,
    Delivered,
}

/// Where one of this device's messages stands (W12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub status: Stage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Typing {
    pub working: bool,
}

/// `127.0.0.1:7878` or a full URL: the base URL of the server.
pub fn base(addr: &str) -> String {
    let a = addr.trim_end_matches('/');
    if a.starts_with("http://") || a.starts_with("https://") {
        a.to_string()
    } else {
        format!("http://{a}")
    }
}
