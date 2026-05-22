use sea_orm_migration::{prelude::*, schema::*};

use super::iden::SystemSettings;

const DEFAULT_CACHE_AFFINITY_TTL_MINUTES: i64 = 5;
const DEFAULT_TOKEN_LIMIT_PER_USER: i64 = 5;

pub(super) fn system_settings_table() -> TableCreateStatement {
    Table::create()
        .table(SystemSettings::Table)
        .if_not_exists()
        .col(string_len(SystemSettings::Id, 36).primary_key())
        .col(string_len(SystemSettings::SiteName, 100))
        .col(string_len(SystemSettings::SiteSubtitle, 200))
        .col(text(SystemSettings::SiteLogoBase64).default(""))
        .col(boolean(SystemSettings::AllowRegistration))
        .col(boolean(SystemSettings::LoginCaptchaEnabled).default(false))
        .col(boolean(SystemSettings::RegistrationCaptchaEnabled).default(false))
        .col(boolean(SystemSettings::SupportTicketCaptchaEnabled).default(true))
        .col(boolean(SystemSettings::RegistrationEmailVerificationEnabled).default(false))
        .col(boolean(SystemSettings::PasswordResetEnabled).default(false))
        .col(boolean(SystemSettings::EmailConfigEnabled).default(false))
        .col(boolean(SystemSettings::SupportTicketEmailNotificationsEnabled).default(false))
        .col(big_integer(SystemSettings::TokenLimitPerUser).default(DEFAULT_TOKEN_LIMIT_PER_USER))
        .col(string_len(SystemSettings::ClientRequestRecordLevel, 20).default("basic"))
        .col(boolean(SystemSettings::ClientRecordRequestHeaders).default(true))
        .col(boolean(SystemSettings::ClientRecordRequestBody).default(true))
        .col(boolean(SystemSettings::ClientRecordResponseHeaders).default(true))
        .col(boolean(SystemSettings::ClientRecordResponseBody).default(true))
        .col(big_integer(SystemSettings::ClientMaxRequestBodySizeKb).default(5120))
        .col(big_integer(SystemSettings::ClientMaxResponseBodySizeKb).default(5120))
        .col(text(SystemSettings::ClientSensitiveRequestHeaders).default("authorization, x-api-key, api-key, cookie, set-cookie"))
        .col(string_len(SystemSettings::ProviderRequestRecordLevel, 20).default("basic"))
        .col(boolean(SystemSettings::ProviderRecordRequestHeaders).default(true))
        .col(boolean(SystemSettings::ProviderRecordRequestBody).default(true))
        .col(boolean(SystemSettings::ProviderRecordResponseHeaders).default(true))
        .col(boolean(SystemSettings::ProviderRecordResponseBody).default(true))
        .col(big_integer(SystemSettings::ProviderMaxRequestBodySizeKb).default(5120))
        .col(big_integer(SystemSettings::ProviderMaxResponseBodySizeKb).default(5120))
        .col(text(SystemSettings::ProviderSensitiveRequestHeaders).default("authorization, x-api-key, api-key, cookie, set-cookie"))
        .col(decimal_len(SystemSettings::DefaultUserGrant, 20, 8))
        .col(big_integer(SystemSettings::DefaultRateLimitRpm))
        .col(string_len(SystemSettings::SchedulingMode, 30))
        .col(big_integer(SystemSettings::CacheAffinityTtlMinutes).default(DEFAULT_CACHE_AFFINITY_TTL_MINUTES))
        .col(text(SystemSettings::ProviderCooldownPolicy).default(r#"{"window_seconds":0,"rules":[]}"#))
        .col(string_len(SystemSettings::SmtpHost, 255))
        .col(big_integer(SystemSettings::SmtpPort).default(587))
        .col(string_len(SystemSettings::SmtpUsername, 255))
        .col(text(SystemSettings::EncryptedSmtpPassword))
        .col(string_len(SystemSettings::SmtpFromEmail, 255))
        .col(string_len(SystemSettings::SmtpFromName, 100))
        .col(string_len(SystemSettings::SmtpEncryption, 20).default("tls"))
        .col(string_len(SystemSettings::EmailSuffixMode, 20).default("none"))
        .col(text(SystemSettings::EmailSuffixes))
        .col(text(SystemSettings::EmailTemplateRegistrationSubject))
        .col(text(SystemSettings::EmailTemplateRegistrationHtml))
        .col(text(SystemSettings::EmailTemplatePasswordResetSubject))
        .col(text(SystemSettings::EmailTemplatePasswordResetHtml))
        .col(timestamp_tz(SystemSettings::CreatedAt))
        .col(timestamp_tz(SystemSettings::UpdatedAt))
        .to_owned()
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}
