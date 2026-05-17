import { spawn } from 'node:child_process';

export async function ensureBackend(serverBaseUrl) {
  if (await healthOk(serverBaseUrl)) {
    return null;
  }
  const child = spawn('cargo', ['run', '-p', 'backend'], {
    cwd: process.cwd(),
    env: process.env,
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let logs = '';
  child.stdout.on('data', (chunk) => {
    logs = keepTail(logs + chunk.toString());
  });
  child.stderr.on('data', (chunk) => {
    logs = keepTail(logs + chunk.toString());
  });
  await waitForHealth(serverBaseUrl, child, () => logs);
  return child;
}

export async function stopBackend(child, serverBaseUrl) {
  if (!child) {
    return;
  }
  child.kill('SIGTERM');
  await waitForDown(serverBaseUrl);
}

async function waitForHealth(serverBaseUrl, child, logs) {
  for (let index = 0; index < 90; index += 1) {
    await sleep(1000);
    if (await healthOk(serverBaseUrl)) {
      return;
    }
    if (child.exitCode !== null) {
      throw new Error(`backend exited before health check passed:\n${logs()}`);
    }
  }
  child.kill('SIGTERM');
  throw new Error(`backend did not become healthy:\n${logs()}`);
}

async function waitForDown(serverBaseUrl) {
  for (let index = 0; index < 50; index += 1) {
    await sleep(200);
    if (!(await healthOk(serverBaseUrl))) {
      return;
    }
  }
  throw new Error(`backend did not stop: ${serverBaseUrl}`);
}

async function healthOk(serverBaseUrl) {
  try {
    const response = await fetch(`${serverBaseUrl}/health`);
    return response.ok;
  } catch {
    return false;
  }
}

function keepTail(value) {
  return value.length > 6000 ? value.slice(value.length - 6000) : value;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
