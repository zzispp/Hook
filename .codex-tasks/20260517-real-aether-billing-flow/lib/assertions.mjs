export function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

export function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

export function assertDecimalEqual(actual, expected, message) {
  const left = Number(actual);
  const right = Number(expected);
  if (!Number.isFinite(left) || !Number.isFinite(right) || Math.abs(left - right) > 0.00000001) {
    throw new Error(`${message}: expected ${expected}, got ${actual}`);
  }
}

export function assertGreaterThan(actual, expected, message) {
  if (!(Number(actual) > Number(expected))) {
    throw new Error(`${message}: expected > ${expected}, got ${actual}`);
  }
}
