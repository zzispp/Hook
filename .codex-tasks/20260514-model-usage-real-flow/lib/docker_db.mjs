import { spawnSync } from 'node:child_process';

export class DockerDb {
  constructor(config) {
    this.container = config.container;
    this.user = config.user;
    this.name = config.name;
  }

  exec(sql) {
    this.run(sql, []);
  }

  scalar(sql) {
    return this.rows(sql)[0]?.[0] ?? '';
  }

  rows(sql) {
    const output = this.run(sql, ['-At', '-F', '\t']);
    if (!output.trim()) return [];
    return output.trimEnd().split('\n').map((line) => line.split('\t'));
  }

  run(sql, extraArgs) {
    const result = spawnSync(
      'docker',
      ['exec', '-i', this.container, 'psql', '-X', '-q', '-v', 'ON_ERROR_STOP=1', ...extraArgs, '-U', this.user, '-d', this.name],
      { input: sql, encoding: 'utf8', maxBuffer: 1024 * 1024 * 20 },
    );
    if (result.status !== 0) {
      throw new Error(`docker psql failed: ${result.stderr || result.stdout}`);
    }
    return result.stdout;
  }
}

export function q(value) {
  if (value === null || value === undefined) return 'null';
  return `'${String(value).replaceAll("'", "''")}'`;
}
