const DECIMAL_TOLERANCE = 0.00000002;

export function expect(condition, message) {
  if (!condition) throw new Error(message);
}

export function expectEqual(actual, expected, message) {
  if (actual !== expected) throw new Error(`${message}: expected ${expected}, got ${actual}`);
}

export function expectIncludes(value, expected, message) {
  expect(String(value).includes(expected), `${message}: missing ${expected}`);
}

export function expectStatus(result, expected, message) {
  expectEqual(result.status, expected, message);
}

export function expectNoTrace(result, message) {
  expectEqual(result.trace.length, 0, message);
}

export function expectSuccess(result, message) {
  expect(result.status >= 200 && result.status < 300, `${message}: HTTP ${result.status} ${result.body}`);
  expect(result.requestId, `${message}: request candidate should be recorded`);
}

export function expectDecimalClose(actual, expected, message) {
  const left = Number(actual);
  const right = Number(expected);
  expect(Number.isFinite(left), `${message}: actual is not numeric: ${actual}`);
  expect(Number.isFinite(right), `${message}: expected is not numeric: ${expected}`);
  expect(Math.abs(left - right) <= DECIMAL_TOLERANCE, `${message}: expected ${expected}, got ${actual}`);
}

export function expectGreaterThan(actual, expected, message) {
  const left = Number(actual);
  expect(Number.isFinite(left), `${message}: value is not numeric: ${actual}`);
  expect(left > expected, `${message}: expected > ${expected}, got ${actual}`);
}

export function rounded(value) {
  return Number(value).toFixed(8);
}

export function providerAttempts(trace, providerName) {
  return trace.filter((row) => row.provider_name === providerName);
}

export function statusCounts(items) {
  return items.reduce((counts, item) => {
    counts[String(item.status)] = (counts[String(item.status)] ?? 0) + 1;
    return counts;
  }, {});
}
