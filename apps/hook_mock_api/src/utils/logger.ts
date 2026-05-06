const COLORS = {
  magenta: '\x1b[35m', // Purple (label)
  cyan: '\x1b[36m', // Cyan (default value)
  yellow: '\x1b[33m', // Yellow (boolean)
  green: '\x1b[32m', // Green (number, bigint)
  red: '\x1b[31m', // Red (null, undefined)
  blue: '\x1b[34m', // Blue (object, array)
  gray: '\x1b[90m', // Gray (symbol, function)
  reset: '\x1b[0m', // Reset to default color
};

function getValueColor(value: unknown): string {
  const type = typeof value;

  switch (true) {
    case value === null || value === undefined:
      return COLORS.red;
    case type === 'number' || type === 'bigint':
      return COLORS.green;
    case type === 'boolean':
      return COLORS.yellow;
    case type === 'object' || Array.isArray(value):
      return COLORS.blue;
    case type === 'symbol' || type === 'function':
      return COLORS.gray;
    default:
      return COLORS.cyan;
  }
}

export function logger(label: string, value: unknown, breakLine: boolean = false): void {
  const type = typeof value;

  const valueColor = getValueColor(value);
  const typeColor = COLORS.gray;

  let formattedValue: string;

  try {
    if (value === null || value === undefined) {
      formattedValue = String(value);
    } else if (type === 'object') {
      formattedValue = JSON.stringify(
        value,
        null,
        breakLine || JSON.stringify(value).length > 50 ? 2 : 0
      );
    } else {
      formattedValue = String(value);
    }
  } catch {
    formattedValue = '[Circular Object]';
  }

  const styleLabel = `🟡 ${COLORS.magenta}${label}:${COLORS.reset}`;
  const styleType = `${typeColor}(${type})${COLORS.reset}`;
  const styleValue = `${valueColor}► ${formattedValue}${COLORS.reset}`;

  console.log(styleLabel, styleType, styleValue);
}

// 🛠️ Test logger
// logger('null', null); // Test null value
// logger('undefined', undefined); // Test undefined value
// logger('string', 'Hello, World!'); // Test string
// logger('number', 42); // Test number
// logger('boolean', true); // Test boolean
// logger('object', { a: 1, b: 2 }, true); // Test object with line breaks
// logger('array', [1, 2, 3]); // Test array
