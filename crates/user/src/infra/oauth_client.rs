use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use types::user::IdentityProvider;

use crate::application::{AppError, AppResult, OAuthClient, OAuthProfile, OAuthProviderSettings};

const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";
const GITHUB_EMAILS_URL: &str = "https://api.github.com/user/emails";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const USER_AGENT: &str = "hook-auth";

#[derive(Clone)]
pub struct ReqwestOAuthClient {
    client: reqwest::Client,
}

impl ReqwestOAuthClient {
    pub fn new() -> AppResult<Self> {
        let client = reqwest::Client::builder().user_agent(USER_AGENT).build().map_err(http_error)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl OAuthClient for ReqwestOAuthClient {
    async fn fetch_profile(&self, provider: IdentityProvider, settings: OAuthProviderSettings, code: &str, redirect_uri: &str) -> AppResult<OAuthProfile> {
        match provider {
            IdentityProvider::Github => self.fetch_github_profile(settings, code, redirect_uri).await,
            IdentityProvider::Google => self.fetch_google_profile(settings, code, redirect_uri).await,
            _ => Err(AppError::InvalidInput("OAuth provider is invalid".into())),
        }
    }
}

impl ReqwestOAuthClient {
    async fn fetch_github_profile(&self, settings: OAuthProviderSettings, code: &str, redirect_uri: &str) -> AppResult<OAuthProfile> {
        let token = self
            .fetch_token(
                GITHUB_TOKEN_URL,
                [
                    ("client_id", settings.client_id.as_str()),
                    ("client_secret", settings.client_secret.as_str()),
                    ("code", code),
                    ("redirect_uri", redirect_uri),
                ],
            )
            .await?;
        let user: GithubUser = self
            .client
            .get(GITHUB_USER_URL)
            .bearer_auth(&token.access_token)
            .send()
            .await
            .map_err(http_error)?
            .error_for_status()
            .map_err(http_error)?
            .json()
            .await
            .map_err(http_error)?;
        let emails: Vec<GithubEmail> = self
            .client
            .get(GITHUB_EMAILS_URL)
            .bearer_auth(&token.access_token)
            .send()
            .await
            .map_err(http_error)?
            .error_for_status()
            .map_err(http_error)?
            .json()
            .await
            .map_err(http_error)?;
        let email = verified_github_email(&emails)?;
        Ok(OAuthProfile {
            subject: user.id.to_string(),
            email,
            email_verified: true,
            display_name: user.name.or(user.login),
            avatar_url: user.avatar_url,
            metadata_json: json_metadata(&user.raw)?,
        })
    }

    async fn fetch_google_profile(&self, settings: OAuthProviderSettings, code: &str, redirect_uri: &str) -> AppResult<OAuthProfile> {
        let token = self
            .fetch_token(
                GOOGLE_TOKEN_URL,
                [
                    ("client_id", settings.client_id.as_str()),
                    ("client_secret", settings.client_secret.as_str()),
                    ("code", code),
                    ("redirect_uri", redirect_uri),
                    ("grant_type", "authorization_code"),
                ],
            )
            .await?;
        let profile: GoogleProfile = self
            .client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(&token.access_token)
            .send()
            .await
            .map_err(http_error)?
            .error_for_status()
            .map_err(http_error)?
            .json()
            .await
            .map_err(http_error)?;
        if !profile.email_verified {
            return Err(AppError::InvalidInput("verified provider email is required".into()));
        }
        Ok(OAuthProfile {
            subject: profile.sub,
            email: profile.email,
            email_verified: profile.email_verified,
            display_name: profile.name,
            avatar_url: profile.picture,
            metadata_json: json_metadata(&profile.raw)?,
        })
    }

    async fn fetch_token<const N: usize>(&self, url: &str, form: [(&str, &str); N]) -> AppResult<OAuthTokenResponse> {
        self.client
            .post(url)
            .header(reqwest::header::ACCEPT, "application/json")
            .form(form.as_slice())
            .send()
            .await
            .map_err(http_error)?
            .error_for_status()
            .map_err(http_error)?
            .json()
            .await
            .map_err(http_error)
    }
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    id: u64,
    login: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
    #[serde(flatten)]
    raw: Value,
}

#[derive(Debug, Deserialize)]
struct GithubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Debug, Deserialize)]
struct GoogleProfile {
    sub: String,
    email: String,
    email_verified: bool,
    name: Option<String>,
    picture: Option<String>,
    #[serde(flatten)]
    raw: Value,
}

fn verified_github_email(emails: &[GithubEmail]) -> AppResult<String> {
    emails
        .iter()
        .find(|item| item.primary && item.verified)
        .or_else(|| emails.iter().find(|item| item.verified))
        .map(|item| item.email.clone())
        .ok_or_else(|| AppError::InvalidInput("verified provider email is required".into()))
}

fn json_metadata(value: &Value) -> AppResult<String> {
    serde_json::to_string(value).map_err(|error| AppError::Infrastructure(format!("OAuth profile serialization error: {error}")))
}

fn http_error(error: reqwest::Error) -> AppError {
    AppError::Infrastructure(format!("OAuth provider request failed: {error}"))
}
