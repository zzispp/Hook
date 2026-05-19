mod email;
mod suffix;

pub(super) use email::{request_registration_email_code, verify_registration_email_code};
pub(super) use suffix::{reject_closed_registration, reject_disallowed_registration_email};
