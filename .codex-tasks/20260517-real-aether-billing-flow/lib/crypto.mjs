import { createCipheriv, createHash, randomBytes } from 'node:crypto';

export function encryptProviderKey(secret, plaintext) {
  const key = createHash('sha256').update(secret).digest();
  const nonce = randomBytes(12);
  const cipher = createCipheriv('aes-256-gcm', key, nonce);
  const encrypted = Buffer.concat([cipher.update(plaintext, 'utf8'), cipher.final(), cipher.getAuthTag()]);
  return `v1:${nonce.toString('hex')}:${encrypted.toString('hex')}`;
}

export function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

export function randomToken(prefix) {
  return `${prefix}-${randomBytes(24).toString('hex')}`;
}
