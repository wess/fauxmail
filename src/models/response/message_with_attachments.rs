//! Response type combining message and attachments.

use crate::models::{attachment::attachment_meta::AttachmentMeta, email::api_email::ApiEmail};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MessageWithAttachments {
    pub message: ApiEmail,
    pub attachments: Vec<AttachmentMeta>,
}
