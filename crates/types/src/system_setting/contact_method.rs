use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactMethodType {
    Wechat,
    Telegram,
    Discord,
    Qq,
    QqGroup,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContactMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub method_type: ContactMethodType,
    pub custom_type: String,
    pub icon: String,
    pub value: String,
    pub qr_code: String,
}
