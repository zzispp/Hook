use serde::Serialize;

use super::{ApiEndpoint, ContactMethod, SystemSettingsResponse};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PublicSiteInfoResponse {
    pub site_name: String,
    pub site_subtitle: String,
    pub public_base_url: String,
    pub site_logo_base64: String,
    pub contact_methods: Vec<ContactMethod>,
    pub api_endpoints: Vec<ApiEndpoint>,
}

impl From<SystemSettingsResponse> for PublicSiteInfoResponse {
    fn from(value: SystemSettingsResponse) -> Self {
        Self {
            site_name: value.site_name,
            site_subtitle: value.site_subtitle,
            public_base_url: value.public_base_url,
            site_logo_base64: value.site_logo_base64,
            contact_methods: value.contact_methods,
            api_endpoints: value.api_endpoints,
        }
    }
}
