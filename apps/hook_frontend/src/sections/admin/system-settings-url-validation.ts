const PUBLIC_BASE_URL_PATTERN =
  /^https?:\/\/((?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?\.)+(?:[A-Za-z]{2,63}|xn--[A-Za-z0-9-]{2,59})|(?:(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])|\[[0-9A-Fa-f:.]+\])(?::[0-9]{1,5})?(?:\/[A-Za-z0-9._~%!$&'()*+,;=:@/-]*)?$/;

export function publicBaseUrlIsValid(value: string) {
  const trimmed = value.trim();
  if (!PUBLIC_BASE_URL_PATTERN.test(trimmed)) {
    return false;
  }
  try {
    const url = new URL(trimmed);
    return publicHostIsValid(url.hostname);
  } catch {
    return false;
  }
}

function publicHostIsValid(hostname: string) {
  const host = hostname.replace(/^\[/, '').replace(/\]$/, '').toLowerCase();
  if (host === 'localhost' || host.endsWith('.localhost')) {
    return false;
  }
  return ipv4IsPublic(host) && ipv6IsPublic(host);
}

function ipv4IsPublic(host: string) {
  if (!/^\d{1,3}(?:\.\d{1,3}){3}$/.test(host)) {
    return true;
  }
  const parts = host.split('.').map(Number);
  const [first, second] = parts;
  return !(
    first === 0 ||
    first === 10 ||
    first === 127 ||
    first >= 224 ||
    (first === 169 && second === 254) ||
    (first === 172 && second >= 16 && second <= 31) ||
    (first === 192 && second === 168) ||
    (first === 255 && second === 255)
  );
}

function ipv6IsPublic(host: string) {
  if (!host.includes(':')) {
    return true;
  }
  const normalized = host.toLowerCase();
  return !(
    normalized === '::' ||
    normalized === '::1' ||
    normalized.startsWith('fc') ||
    normalized.startsWith('fd') ||
    normalized.startsWith('fe8') ||
    normalized.startsWith('fe9') ||
    normalized.startsWith('fea') ||
    normalized.startsWith('feb') ||
    normalized.startsWith('ff')
  );
}
