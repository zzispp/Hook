import net from 'node:net';

export class RedisClient {
  constructor(config) {
    this.host = config.host;
    this.port = config.port;
  }

  async command(...args) {
    const response = await rawCommand(this.host, this.port, args);
    return parseResponse(response);
  }

  async del(...keys) {
    if (keys.length === 0) {
      return 0;
    }
    return this.command('DEL', ...keys);
  }

  async hdel(key, ...fields) {
    if (fields.length === 0) {
      return 0;
    }
    return this.command('HDEL', key, ...fields);
  }

  async hlen(key) {
    return this.command('HLEN', key);
  }

  async keys(pattern) {
    return this.command('KEYS', pattern);
  }
}

function rawCommand(host, port, args) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection({ host, port });
    const chunks = [];
    socket.setTimeout(5000);
    socket.on('connect', () => socket.write(renderCommand(args)));
    socket.on('data', (chunk) => {
      chunks.push(chunk);
      const buffer = Buffer.concat(chunks);
      if (tryParse(buffer)) {
        socket.end();
        resolve(buffer);
      }
    });
    socket.on('timeout', () => {
      socket.destroy();
      reject(new Error('redis command timed out'));
    });
    socket.on('error', reject);
  });
}

function renderCommand(args) {
  return `*${args.length}\r\n${args.map((arg) => `$${Buffer.byteLength(String(arg))}\r\n${arg}\r\n`).join('')}`;
}

function parseResponse(buffer) {
  const parsed = parseValue(buffer, 0);
  if (!parsed) {
    throw new Error('incomplete redis response');
  }
  if (parsed.error) {
    throw new Error(parsed.error);
  }
  return parsed.value;
}

function tryParse(buffer) {
  try {
    return Boolean(parseValue(buffer, 0));
  } catch {
    return true;
  }
}

function parseValue(buffer, offset) {
  const prefix = String.fromCharCode(buffer[offset]);
  if (prefix === '+') return parseLine(buffer, offset, (line) => line);
  if (prefix === '-') return parseLine(buffer, offset, (line) => ({ error: line }));
  if (prefix === ':') return parseLine(buffer, offset, (line) => Number(line));
  if (prefix === '$') return parseBulk(buffer, offset);
  if (prefix === '*') return parseArray(buffer, offset);
  throw new Error(`unsupported redis response prefix: ${prefix}`);
}

function parseLine(buffer, offset, map) {
  const end = buffer.indexOf('\r\n', offset);
  if (end < 0) {
    return null;
  }
  const line = buffer.subarray(offset + 1, end).toString();
  const value = map(line);
  return value?.error ? value : { value, next: end + 2 };
}

function parseBulk(buffer, offset) {
  const head = parseLine(buffer, offset, Number);
  if (!head) {
    return null;
  }
  if (head.value < 0) {
    return { value: null, next: head.next };
  }
  const end = head.next + head.value;
  if (buffer.length < end + 2) {
    return null;
  }
  return { value: buffer.subarray(head.next, end).toString(), next: end + 2 };
}

function parseArray(buffer, offset) {
  const head = parseLine(buffer, offset, Number);
  if (!head) {
    return null;
  }
  const values = [];
  let next = head.next;
  for (let index = 0; index < head.value; index += 1) {
    const item = parseValue(buffer, next);
    if (!item) {
      return null;
    }
    values.push(item.value);
    next = item.next;
  }
  return { value: values, next };
}
