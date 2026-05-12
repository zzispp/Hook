import { spawnSync } from 'node:child_process';

export class Db {
  constructor(config) {
    this.config = config;
  }

  exec(sql) {
    return this.run(sql, []);
  }

  scalar(sql) {
    return this.rows(sql)[0]?.[0] ?? '';
  }

  rows(sql) {
    const output = this.run(sql, ['-At', '-F', '\t']);
    if (!output.trim()) {
      return [];
    }
    return output.trimEnd().split('\n').map((line) => line.split('\t'));
  }

  run(sql, extraArgs) {
    const c = this.config;
    const result = spawnSync(c.psqlBin, ['-X', '-q', '-v', 'ON_ERROR_STOP=1', ...extraArgs, '-h', c.host, '-p', c.port, '-U', c.user, '-d', c.name], {
      input: sql,
      encoding: 'utf8',
      env: { ...process.env, PGPASSWORD: c.password },
      maxBuffer: 1024 * 1024 * 20,
    });
    if (result.status !== 0) {
      throw new Error(`psql failed: ${result.stderr || result.stdout}`);
    }
    return result.stdout;
  }
}

export function q(value) {
  if (value === null || value === undefined) {
    return 'null';
  }
  return `'${String(value).replaceAll("'", "''")}'`;
}

