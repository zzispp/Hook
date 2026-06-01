use serde::Serialize;

use super::{ContactMethod, SystemSettingsResponse};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PublicSiteInfoResponse {
    pub site_name: String,
    pub site_subtitle: String,
    pub site_logo_base64: String,
    pub contact_methods: Vec<ContactMethod>,
}

impl From<SystemSettingsResponse> for PublicSiteInfoResponse {
    fn from(value: SystemSettingsResponse) -> Self {
        Self {
            site_name: value.site_name,
            site_subtitle: value.site_subtitle,
            site_logo_base64: value.site_logo_base64,
            contact_methods: value.contact_methods,
        }
    }
}
