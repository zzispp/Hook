use std::{net::IpAddr, sync::LazyLock};

use regex::Regex;

const PUBLIC_BASE_URL_PATTERN: &str = concat!(
    r"^https?://(?P<host>",
    r"(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?\.)+(?:[A-Za-z]{2,63}|xn--[A-Za-z0-9-]{2,59})",
    r"|(?:(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])",
    r"|\[[0-9A-Fa-f:.]+\]",
    r")(?::[0-9]{1,5})?(?:/[A-Za-z0-9._~%!$&'()*+,;=:@/-]*)?$",
);
static PUBLIC_BASE_URL_REGEX: LazyLock<Result<Regex, regex::Error>> = LazyLock::new(|| Regex::new(PUBLIC_BASE_URL_PATTERN));

pub fn public_base_url_is_valid(value: &str) -> Result<bool, String> {
    let regex = PUBLIC_BASE_URL_REGEX.as_ref().map_err(|error| error.to_string())?;
    let Some(captures) = regex.captures(value) else {
        return Ok(false);
    };
    let Some(host) = captures.name("host") else {
        return Ok(false);
    };
    Ok(public_host_is_valid(host.as_str()))
}

pub fn public_base_url_domain(value: &str) -> Result<String, String> {
    let url = url::Url::parse(value).map_err(|error| error.to_string())?;
    let host = url.host_str().ok_or_else(|| "public base URL host is required".to_owned())?;
    let host = public_base_url_authority_host(host);
    Ok(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host,
    })
}

fn public_base_url_authority_host(host: &str) -> String {
    if host.contains(':') {
        return format!("[{host}]");
    }
    host.to_owned()
}

fn public_host_is_valid(host: &str) -> bool {
    let normalized = host.trim_start_matches('[').trim_end_matches(']').to_ascii_lowercase();
    if normalized == "localhost" || normalized.ends_with(".localhost") {
        return false;
    }
    normalized.parse::<IpAddr>().map_or(true, public_ip_is_valid)
}

fn public_ip_is_valid(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => {
            !(addr.is_private() || addr.is_loopback() || addr.is_link_local() || addr.is_broadcast() || addr.is_unspecified() || addr.is_multicast())
        }
        IpAddr::V6(addr) => !(addr.is_loopback() || addr.is_unspecified() || addr.is_unique_local() || addr.is_multicast() || ipv6_is_link_local(addr)),
    }
}

fn ipv6_is_link_local(addr: std::net::Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::{public_base_url_domain, public_base_url_is_valid};

    #[test]
    fn public_base_url_accepts_http_and_https_urls() {
        assert_eq!(public_base_url_is_valid("http://hook.test").unwrap(), true);
        assert_eq!(public_base_url_is_valid("https://hook.test/payment").unwrap(), true);
        assert_eq!(public_base_url_is_valid("https://1.1.1.1/callback").unwrap(), true);
    }

    #[test]
    fn public_base_url_rejects_missing_or_unsupported_scheme() {
        assert_eq!(public_base_url_is_valid("hook.test").unwrap(), false);
        assert_eq!(public_base_url_is_valid("ftp://hook.test").unwrap(), false);
        assert_eq!(public_base_url_is_valid("https://").unwrap(), false);
    }

    #[test]
    fn public_base_url_rejects_local_and_private_hosts() {
        assert_eq!(public_base_url_is_valid("http://localhost:8080").unwrap(), false);
        assert_eq!(public_base_url_is_valid("http://127.0.0.1").unwrap(), false);
        assert_eq!(public_base_url_is_valid("http://10.0.0.1").unwrap(), false);
        assert_eq!(public_base_url_is_valid("http://[::1]").unwrap(), false);
    }

    #[test]
    fn public_base_url_domain_uses_url_authority() {
        assert_eq!(public_base_url_domain("https://hook.test/app").unwrap(), "hook.test");
        assert_eq!(public_base_url_domain("https://hook.test:8443/app").unwrap(), "hook.test:8443");
    }
}
