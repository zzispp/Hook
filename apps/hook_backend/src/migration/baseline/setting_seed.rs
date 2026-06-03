use sea_orm_migration::prelude::*;

use super::iden::SystemSettings;

const SYSTEM_SETTINGS_ID: &str = "global";
const DEFAULT_CACHE_AFFINITY_TTL_MINUTES: i64 = 5;
const DEFAULT_TOKEN_LIMIT_PER_USER: i64 = 5;
const DEFAULT_RECHARGE_EXPIRE_MINUTES: i64 = 15;
const DEFAULT_RECHARGE_MAX_UNPAID_ORDERS: i64 = 5;
const DEFAULT_REGISTRATION_SUBJECT: &str = "注册验证码";
const DEFAULT_PASSWORD_RESET_SUBJECT: &str = "找回密码";
const DEFAULT_REGISTRATION_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<body style="margin:0;padding:0;background:#F9FAFB;font-family:'Public Sans',Arial,'Microsoft YaHei',sans-serif;color:#1C252E;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#F9FAFB;padding:32px 16px;">
    <tr>
      <td align="center">
        <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width:560px;background:#FFFFFF;border:1px solid #DFE3E8;border-radius:8px;overflow:hidden;">
          <tr><td style="height:6px;background:#00A76F;"></td></tr>
          <tr>
            <td style="padding:32px;">
              <p style="margin:0 0 16px;color:#007867;font-size:13px;font-weight:700;letter-spacing:0;">{{app_name}}</p>
              <h1 style="margin:0 0 12px;color:#1C252E;font-size:24px;line-height:1.35;font-weight:700;">注册验证码</h1>
              <p style="margin:0 0 24px;color:#637381;font-size:15px;line-height:1.7;">请使用以下验证码完成邮箱验证。</p>
              <div style="padding:20px 16px;background:#C8FAD6;border:1px solid #5BE49B;border-radius:8px;text-align:center;">
                <span style="color:#004B50;font-size:36px;line-height:1.2;font-weight:700;letter-spacing:8px;">{{code}}</span>
              </div>
              <p style="margin:24px 0 0;color:#637381;font-size:14px;line-height:1.7;">验证码将在 {{expire_minutes}} 分钟后失效。</p>
              <p style="margin:8px 0 0;color:#919EAB;font-size:13px;line-height:1.7;">收件邮箱：{{email}}</p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#;
const DEFAULT_PASSWORD_RESET_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<body style="margin:0;padding:0;background:#F9FAFB;font-family:'Public Sans',Arial,'Microsoft YaHei',sans-serif;color:#1C252E;">
  <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="background:#F9FAFB;padding:32px 16px;">
    <tr>
      <td align="center">
        <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="max-width:560px;background:#FFFFFF;border:1px solid #DFE3E8;border-radius:8px;overflow:hidden;">
          <tr><td style="height:6px;background:#00A76F;"></td></tr>
          <tr>
            <td style="padding:32px;">
              <p style="margin:0 0 16px;color:#007867;font-size:13px;font-weight:700;letter-spacing:0;">{{app_name}}</p>
              <h1 style="margin:0 0 12px;color:#1C252E;font-size:24px;line-height:1.35;font-weight:700;">找回密码</h1>
              <p style="margin:0 0 24px;color:#637381;font-size:15px;line-height:1.7;">请点击下方按钮继续重置账户密码。</p>
              <p style="margin:0 0 24px;">
                <a href="{{reset_link}}" style="display:inline-block;padding:12px 22px;background:#00A76F;color:#FFFFFF;text-decoration:none;border-radius:8px;font-size:14px;font-weight:700;">重置密码</a>
              </p>
              <p style="margin:0 0 12px;color:#637381;font-size:14px;line-height:1.7;">链接将在 {{expire_minutes}} 分钟后失效。</p>
              <p style="margin:0;color:#919EAB;font-size:13px;line-height:1.7;">无法打开按钮时，请复制链接访问：{{reset_link}}</p>
              <p style="margin:8px 0 0;color:#919EAB;font-size:13px;line-height:1.7;">收件邮箱：{{email}}</p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#;

const DEFAULT_SITE_LOGO_BASE64: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAzMiAzMiI+CiAgPHN0eWxlPgogICAgcGF0aCB7IGZpbGw6ICMwMDAwMDA7IH0KICAgIEBtZWRpYSAocHJlZmVycy1jb2xvci1zY2hlbWU6IGRhcmspIHsKICAgICAgcGF0aCB7IGZpbGw6ICNGRkZGRkY7IH0KICAgIH0KICA8L3N0eWxlPgogIDxwYXRoIGQ9Ik0xMS4wNSA3LjAyNThIMTQuNTUyTDEzLjUxOTQgMTMuMjE4NkgxNi4zMTczTDIwLjEyMzQgMTguMjQxMUgxMi42ODE1TDExLjk2MzcgMjIuNTQ5OUg4LjQ2MTdMMTEuMDUgNy4wMjU4WiBNMjMuNDUgNy4wMjU4SDI2Ljk1MkwyNC4zNjM3IDIyLjU0OTlIMjAuODYxN0wyMS41Nzk1IDE4LjI0MTFIMjEuOTkyNkwxOC4xODY1IDEzLjIxODZIMjIuNDE3NEwyMy40NSA3LjAyNThaIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiLz4KPC9zdmc+Cg==";

pub(super) async fn seed_system_settings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager.execute(system_settings_insert()).await
}

fn system_settings_insert() -> InsertStatement {
    Query::insert()
        .into_table(SystemSettings::Table)
        .columns(system_settings_columns())
        .values_panic(system_settings_values())
        .to_owned()
}

fn system_settings_columns() -> Vec<SystemSettings> {
    vec![
        SystemSettings::Id,
        SystemSettings::SiteName,
        SystemSettings::SiteSubtitle,
        SystemSettings::PublicBaseUrl,
        SystemSettings::SiteLogoBase64,
        SystemSettings::ContactMethods,
        SystemSettings::AllowRegistration,
        SystemSettings::LoginCaptchaEnabled,
        SystemSettings::RegistrationCaptchaEnabled,
        SystemSettings::SupportTicketCaptchaEnabled,
        SystemSettings::RechargeCaptchaEnabled,
        SystemSettings::RegistrationEmailVerificationEnabled,
        SystemSettings::AuthGithubEnabled,
        SystemSettings::AuthGithubClientId,
        SystemSettings::EncryptedAuthGithubClientSecret,
        SystemSettings::AuthGoogleEnabled,
        SystemSettings::AuthGoogleClientId,
        SystemSettings::EncryptedAuthGoogleClientSecret,
        SystemSettings::AuthEvmEnabled,
        SystemSettings::AuthEvmChainIds,
        SystemSettings::AuthEvmStatement,
        SystemSettings::PasswordResetEnabled,
        SystemSettings::EmailConfigEnabled,
        SystemSettings::SupportTicketEmailNotificationsEnabled,
        SystemSettings::DefaultUserGroupCode,
        SystemSettings::TokenLimitPerUser,
        SystemSettings::ClientRequestRecordLevel,
        SystemSettings::ClientRecordRequestHeaders,
        SystemSettings::ClientRecordRequestBody,
        SystemSettings::ClientRecordResponseHeaders,
        SystemSettings::ClientRecordResponseBody,
        SystemSettings::ClientMaxRequestBodySizeKb,
        SystemSettings::ClientMaxResponseBodySizeKb,
        SystemSettings::ClientSensitiveRequestHeaders,
        SystemSettings::ProviderRequestRecordLevel,
        SystemSettings::ProviderRecordRequestHeaders,
        SystemSettings::ProviderRecordRequestBody,
        SystemSettings::ProviderRecordResponseHeaders,
        SystemSettings::ProviderRecordResponseBody,
        SystemSettings::ProviderMaxRequestBodySizeKb,
        SystemSettings::ProviderMaxResponseBodySizeKb,
        SystemSettings::ProviderSensitiveRequestHeaders,
        SystemSettings::DefaultUserGrant,
        SystemSettings::DefaultRateLimitRpm,
        SystemSettings::RechargeEnabled,
        SystemSettings::RechargeArrivalRatio,
        SystemSettings::RechargeOrderExpireMinutes,
        SystemSettings::RechargeMaxUnpaidOrders,
        SystemSettings::RechargeMinAmount,
        SystemSettings::RechargeMaxAmount,
        SystemSettings::SchedulingMode,
        SystemSettings::ProviderPriorityMode,
        SystemSettings::KeyPrioritySnapshotInitialized,
        SystemSettings::CacheAffinityTtlMinutes,
        SystemSettings::ProviderCooldownPolicy,
        SystemSettings::SmtpHost,
        SystemSettings::SmtpPort,
        SystemSettings::SmtpUsername,
        SystemSettings::EncryptedSmtpPassword,
        SystemSettings::SmtpFromEmail,
        SystemSettings::SmtpFromName,
        SystemSettings::SmtpEncryption,
        SystemSettings::EmailSuffixMode,
        SystemSettings::EmailSuffixes,
        SystemSettings::EmailTemplateRegistrationSubject,
        SystemSettings::EmailTemplateRegistrationHtml,
        SystemSettings::EmailTemplatePasswordResetSubject,
        SystemSettings::EmailTemplatePasswordResetHtml,
        SystemSettings::CreatedAt,
        SystemSettings::UpdatedAt,
    ]
}

fn system_settings_values() -> Vec<Expr> {
    vec![
        SYSTEM_SETTINGS_ID.into(),
        "Hook AI".into(),
        "自托管 AI API 统一网关".into(),
        "".into(),
        DEFAULT_SITE_LOGO_BASE64.into(),
        "[]".into(),
        true.into(),
        false.into(),
        false.into(),
        true.into(),
        false.into(),
        false.into(),
        false.into(),
        "".into(),
        "".into(),
        false.into(),
        "".into(),
        "".into(),
        false.into(),
        "1".into(),
        "Sign in to Hook".into(),
        false.into(),
        false.into(),
        false.into(),
        constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
        DEFAULT_TOKEN_LIMIT_PER_USER.into(),
        "full".into(),
        true.into(),
        true.into(),
        true.into(),
        true.into(),
        5120.into(),
        5120.into(),
        "authorization, x-api-key, api-key, cookie, set-cookie".into(),
        "full".into(),
        true.into(),
        true.into(),
        true.into(),
        true.into(),
        5120.into(),
        5120.into(),
        "authorization, x-api-key, api-key, cookie, set-cookie".into(),
        0.into(),
        0.into(),
        false.into(),
        1.into(),
        DEFAULT_RECHARGE_EXPIRE_MINUTES.into(),
        DEFAULT_RECHARGE_MAX_UNPAID_ORDERS.into(),
        Expr::cust("0.01"),
        3000.into(),
        "cache_affinity".into(),
        "provider".into(),
        false.into(),
        DEFAULT_CACHE_AFFINITY_TTL_MINUTES.into(),
        r#"{"window_seconds":0,"rules":[]}"#.into(),
        "".into(),
        587.into(),
        "".into(),
        "".into(),
        "".into(),
        "Hook".into(),
        "tls".into(),
        "none".into(),
        "".into(),
        DEFAULT_REGISTRATION_SUBJECT.into(),
        DEFAULT_REGISTRATION_HTML.into(),
        DEFAULT_PASSWORD_RESET_SUBJECT.into(),
        DEFAULT_PASSWORD_RESET_HTML.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

#[cfg(test)]
mod tests {
    use sea_orm_migration::prelude::Value;

    use super::*;

    #[test]
    fn system_settings_seed_values_match_auth_provider_columns() {
        let columns = system_settings_columns();
        let values = system_settings_values();

        assert_eq!(columns.len(), values.len());
        assert_bool_value(&columns, &values, SystemSettings::AuthGithubEnabled, false);
        assert_string_value(&columns, &values, SystemSettings::AuthGithubClientId, "");
        assert_string_value(&columns, &values, SystemSettings::EncryptedAuthGithubClientSecret, "");
        assert_bool_value(&columns, &values, SystemSettings::AuthGoogleEnabled, false);
        assert_string_value(&columns, &values, SystemSettings::AuthGoogleClientId, "");
        assert_string_value(&columns, &values, SystemSettings::EncryptedAuthGoogleClientSecret, "");
    }

    fn assert_bool_value(columns: &[SystemSettings], values: &[Expr], column: SystemSettings, expected: bool) {
        let index = column_index(columns, column);

        assert_eq!(values[index], Expr::Value(Value::Bool(Some(expected))));
    }

    fn assert_string_value(columns: &[SystemSettings], values: &[Expr], column: SystemSettings, expected: &str) {
        let index = column_index(columns, column);

        assert_eq!(values[index], Expr::Value(Value::String(Some(expected.to_owned()))));
    }

    fn column_index(columns: &[SystemSettings], column: SystemSettings) -> usize {
        columns
            .iter()
            .position(|candidate| candidate.to_string() == column.to_string())
            .expect("seed column exists")
    }
}
