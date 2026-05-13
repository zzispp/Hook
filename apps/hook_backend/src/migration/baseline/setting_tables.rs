use sea_orm_migration::{prelude::*, schema::*};

use super::iden::SystemSettings;

pub(super) fn system_settings_table() -> TableCreateStatement {
    Table::create()
        .table(SystemSettings::Table)
        .if_not_exists()
        .col(string_len(SystemSettings::Id, 36).primary_key())
        .col(string_len(SystemSettings::SiteName, 100))
        .col(string_len(SystemSettings::SiteSubtitle, 200))
        .col(boolean(SystemSettings::AllowRegistration))
        .col(boolean(SystemSettings::LoginCaptchaEnabled).default(false))
        .col(boolean(SystemSettings::RegistrationCaptchaEnabled).default(false))
        .col(boolean(SystemSettings::RegistrationEmailVerificationEnabled).default(false))
        .col(boolean(SystemSettings::EmailConfigEnabled).default(false))
        .col(boolean(SystemSettings::AutoDeleteExpiredTokens))
        .col(big_integer(SystemSettings::RequestRecordRetentionDays))
        .col(big_integer(SystemSettings::RequestRecordPayloadRetentionDays))
        .col(string_len(SystemSettings::RequestRecordLevel, 20).default("basic"))
        .col(big_integer(SystemSettings::MaxRequestBodySizeKb).default(5120))
        .col(big_integer(SystemSettings::MaxResponseBodySizeKb).default(5120))
        .col(text(SystemSettings::SensitiveRequestHeaders).default("authorization, x-api-key, api-key, cookie, set-cookie"))
        .col(boolean(SystemSettings::RecordRequestHeaders).default(false))
        .col(boolean(SystemSettings::RecordRequestBody).default(false))
        .col(boolean(SystemSettings::RecordResponseBody).default(false))
        .col(decimal_len(SystemSettings::DefaultUserGrant, 20, 8))
        .col(big_integer(SystemSettings::DefaultRateLimitRpm))
        .col(string_len(SystemSettings::SchedulingMode, 30))
        .col(string_len(SystemSettings::Currency, 3).default("USD"))
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
