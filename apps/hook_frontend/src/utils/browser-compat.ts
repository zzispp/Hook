const UUID_BYTE_LENGTH = 16;
const UUID_VERSION_INDEX = 6;
const UUID_VARIANT_INDEX = 8;
const UUID_VERSION_RANGE = 0x10;
const UUID_VERSION_BITS = 0x40;
const UUID_VARIANT_RANGE = 0x40;
const UUID_VARIANT_BITS = 0x80;
const UUID_GROUP_SIZES = [4, 2, 2, 2, 6] as const;
const HEX_RADIX = 16;
const HEX_BYTE_WIDTH = 2;

export function createUuid() {
  const cryptoApi = globalThis.crypto;
  if (typeof cryptoApi?.randomUUID === 'function') {
    return cryptoApi.randomUUID();
  }
  if (typeof cryptoApi?.getRandomValues !== 'function') {
    throw new Error('Secure UUID generation is unavailable in this browser.');
  }

  const bytes = cryptoApi.getRandomValues(new Uint8Array(UUID_BYTE_LENGTH));
  bytes[UUID_VERSION_INDEX] = (bytes[UUID_VERSION_INDEX] % UUID_VERSION_RANGE) + UUID_VERSION_BITS;
  bytes[UUID_VARIANT_INDEX] = (bytes[UUID_VARIANT_INDEX] % UUID_VARIANT_RANGE) + UUID_VARIANT_BITS;
  return formatUuid(bytes);
}

export async function copyText(value: string) {
  let nativeError: unknown;
  const clipboard = globalThis.navigator?.clipboard;

  if (typeof clipboard?.writeText === 'function') {
    try {
      await clipboard.writeText(value);
      return;
    } catch (error) {
      nativeError = error;
    }
  }

  try {
    copyTextWithLegacyCommand(value);
  } catch (legacyError) {
    throw clipboardCopyError(nativeError, legacyError);
  }
}

function formatUuid(bytes: Uint8Array) {
  const hexBytes = Array.from(bytes, (byte) => byte.toString(HEX_RADIX).padStart(HEX_BYTE_WIDTH, '0'));
  let offset = 0;
  return UUID_GROUP_SIZES.map((size) => {
    const group = hexBytes.slice(offset, offset + size).join('');
    offset += size;
    return group;
  }).join('-');
}

function copyTextWithLegacyCommand(value: string) {
  const currentDocument = globalThis.document;
  if (!currentDocument?.body || typeof currentDocument.execCommand !== 'function') {
    throw new Error('Legacy clipboard API is unavailable.');
  }

  const textarea = currentDocument.createElement('textarea');
  textarea.value = value;
  textarea.setAttribute('readonly', '');
  textarea.style.position = 'fixed';
  textarea.style.left = '-9999px';
  textarea.style.opacity = '0';
  currentDocument.body.appendChild(textarea);

  try {
    textarea.select();
    if (!currentDocument.execCommand('copy')) {
      throw new Error('Legacy clipboard command was rejected.');
    }
  } finally {
    textarea.remove();
  }
}

function clipboardCopyError(nativeError: unknown, legacyError: unknown) {
  const causes = [nativeError, legacyError].filter(Boolean).map(errorMessage).join('; ');
  return new Error(`Clipboard copy failed: ${causes}`);
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
