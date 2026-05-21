export const DEFAULT_REGISTRATION_TEMPLATE_HTML = `<!DOCTYPE html>
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
</html>`;

export const DEFAULT_PASSWORD_RESET_TEMPLATE_HTML = `<!DOCTYPE html>
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
</html>`;
