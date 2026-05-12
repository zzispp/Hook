import { spawn } from 'node:child_process';

export async function ensureBackend(serverBaseUrl) {
  if (await healthOk(serverBaseUrl)) {
    console.log(`backend: using existing ${serverBaseUrl}`);
    return null;
  }
  console.log('backend: starting cargo run -p backend');
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
  console.log('backend: started');
  return child;
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

