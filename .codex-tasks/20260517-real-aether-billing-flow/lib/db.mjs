import { spawnSync } from 'node:child_process';

export class DockerDb {
  constructor(config) {
    this.config = config;
  }

  exec(sql) {
    this.run(sql, []);
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
    const { container, user, name } = this.config;
    const result = spawnSync(
      'docker',
      ['exec', '-i', container, 'psql', '-X', '-q', '-v', 'ON_ERROR_STOP=1', ...extraArgs, '-U', user, '-d', name],
      { input: sql, encoding: 'utf8', maxBuffer: 1024 * 1024 * 20 },
    );
    if (result.status !== 0) {
      throw new Error(`docker psql failed: ${result.stderr || result.stdout}`);
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
