use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityProvider {
    Github,
    Google,
    Evm,
}

impl IdentityProvider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Google => "google",
            Self::Evm => "evm",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Github => "GitHub",
            Self::Google => "Google",
            Self::Evm => "EVM",
        }
    }
}

impl TryFrom<&str> for IdentityProvider {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "github" => Ok(Self::Github),
            "google" => Ok(Self::Google),
            "evm" => Ok(Self::Evm),
            _ => Err(format!("invalid identity provider: {value}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UserIdentity {
    pub id: String,
    pub user_id: String,
    pub provider: IdentityProvider,
    pub provider_subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserIdentityInput {
    pub user_id: String,
    pub provider: IdentityProvider,
    pub provider_subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub metadata_json: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UserIdentitySummary {
    pub id: String,
    pub provider: IdentityProvider,
    pub provider_subject: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

impl From<UserIdentity> for UserIdentitySummary {
    fn from(value: UserIdentity) -> Self {
        Self {
            id: value.id,
            provider: value.provider,
            provider_subject: value.provider_subject,
            email: value.email,
            email_verified: value.email_verified,
            display_name: value.display_name,
            avatar_url: value.avatar_url,
            created_at: value.created_at,
            last_login_at: value.last_login_at,
        }
    }
}
