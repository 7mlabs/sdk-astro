'use strict';
const { createEngineBridge } = require('./bridge.cjs');
const { readFileSync, readdirSync, statSync } = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');

const OPERATIONS = Object.freeze(['chart', 'harmonic', 'synastry', 'natal', 'natalDomains',
  'couple', 'composite', 'events', 'forecast', 'query']);
const MAX_INPUT_BYTES = 4 * 1024 * 1024;

/** Resolve an installed SDK, never a source binding or an uninstalled tarball. */
function resolveEngineModule(options = {}) {
  if (options.engineModule || process.env.ASTRO_ENGINE_MODULE)
    return options.engineModule || process.env.ASTRO_ENGINE_MODULE;
  try { return require.resolve('@7mlabs/astrology', { paths: [options.baseDirectory || process.cwd(), __dirname] }); }
  catch { /* A development checkout may have only the verified artifact consumers. */ }
  const engineRoot = path.resolve(options.engineDirectory || path.join(__dirname, '../..'));
  try {
    const packages = path.join(engineRoot, 'artifacts/packages');
    const manifest = JSON.parse(readFileSync(path.join(packages, 'manifest.json'), 'utf8'));
    if (manifest.platform !== process.platform || manifest.arch !== process.arch) return '@7mlabs/astrology';
    const filename = Object.keys(manifest.files).find(name => /^7mlabs-astrology-.*\.tgz$/.test(name));
    if (!filename || path.basename(filename) !== filename) return '@7mlabs/astrology';
    const artifact = readFileSync(path.join(packages, filename));
    if (createHash('sha256').update(artifact).digest('hex') !== manifest.files[filename]) return '@7mlabs/astrology';
    const integrity = `sha512-${createHash('sha512').update(artifact).digest('base64')}`;
    // Match npm's installed lockfile to the current checked artifact. Folder
    // names and modification times are not a guarantee of the installed build.
    const consumers = path.join(engineRoot, 'artifacts/consumers');
    const candidates = readdirSync(consumers, { withFileTypes: true }).filter(entry => entry.isDirectory())
      .map(entry => path.join(consumers, entry.name, 'node')).filter(directory => {
        try {
          const lock = JSON.parse(readFileSync(path.join(directory, 'package-lock.json'), 'utf8'));
          const installed = lock.packages?.['node_modules/@7mlabs/astrology'];
          return installed?.version === manifest.engineVersion && installed.integrity === integrity;
        } catch { return false; }
      }).sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs);
    for (const directory of candidates) {
      try { return require.resolve('@7mlabs/astrology', { paths: [directory] }); }
      catch { /* A partial installation is not usable. */ }
    }
  } catch { /* Missing build/install artifacts do not break the static website. */ }
  return '@7mlabs/astrology';
}

class HostError extends Error {
  constructor(status, code, message) { super(message); this.status = status; this.code = code; }
}
function sendJson(res, status, value, headers = {}) {
  if (res.destroyed || res.writableEnded) return;
  res.writeHead(status, { 'Content-Type': 'application/json; charset=utf-8',
    'Cache-Control': 'no-store', 'X-Content-Type-Options': 'nosniff', ...headers });
  res.end(JSON.stringify(value));
}
function checkLocalRequest(req) {
  const address = req.socket.remoteAddress;
  if (!['127.0.0.1', '::1', '::ffff:127.0.0.1'].includes(address))
    throw new HostError(403, 'LOCAL_ONLY', 'The native runtime accepts loopback requests only.');
  const host = req.headers.host;
  const port = req.socket.localPort;
  if (typeof host !== 'string' || ![`127.0.0.1:${port}`, `localhost:${port}`].includes(host))
    throw new HostError(403, 'FORBIDDEN_HOST', 'Use the current loopback host and port.');
  const origin = req.headers.origin;
  if (origin !== undefined && origin !== `http://${host}`)
    throw new HostError(403, 'FORBIDDEN_ORIGIN', 'Only this local UI origin may call the runtime.');
  const site = req.headers['sec-fetch-site'];
  if (site !== undefined && !['same-origin', 'none'].includes(site))
    throw new HostError(403, 'FORBIDDEN_ORIGIN', 'Cross-site runtime requests are not allowed.');
}
function readRawBody(req, maximum) {
  const declared = req.headers['content-length'];
  if (declared !== undefined && (!/^\d+$/.test(declared) || Number(declared) > maximum)) {
    req.resume();
    return Promise.reject(new HostError(413, 'INPUT_TOO_LARGE', 'Request exceeds the local runtime input limit.'));
  }
  return new Promise((resolve, reject) => {
    let chunks = [], bytes = 0, settled = false;
    const fail = error => { if (!settled) { settled = true; chunks = []; reject(error); } };
    req.on('data', chunk => {
      if (settled) return;
      bytes += chunk.length;
      if (bytes > maximum) return fail(new HostError(413, 'INPUT_TOO_LARGE', 'Request exceeds the local runtime input limit.'));
      chunks.push(chunk);
    });
    req.once('aborted', () => fail(new HostError(400, 'REQUEST_ABORTED', 'The request ended before its body was complete.')));
    req.once('error', () => fail(new HostError(400, 'REQUEST_ABORTED', 'The request body could not be read.')));
    req.once('end', () => {
      if (settled) return;
      settled = true;
      try { resolve(new TextDecoder('utf-8', { fatal: true }).decode(Buffer.concat(chunks, bytes))); }
      catch { reject(new HostError(400, 'INVALID_ENCODING', 'Request JSON must use valid UTF-8.')); }
    });
  });
}

/** Reusable same-origin API; call handle before a static server's normal routing. */
function createLocalEngineHost(options = {}) {
  const maxInputBytes = options.maxInputBytes ?? MAX_INPUT_BYTES;
  const maxPendingRequests = options.maxPendingRequests ?? 8;
  if (!Number.isInteger(maxInputBytes) || maxInputBytes < 1 || maxInputBytes > MAX_INPUT_BYTES)
    throw new RangeError('maxInputBytes must be an integer between1 and4194304');
  if (!Number.isInteger(maxPendingRequests) || maxPendingRequests < 1 || maxPendingRequests > 32)
    throw new RangeError('maxPendingRequests must be an integer between1 and32');
  const bridge = createEngineBridge({ engineModule: resolveEngineModule(options) });
  let available = false, engineVersion = null, reason = null, inFlightRequests = 0, closed = false;
  const snapshot = () => ({ available: available && !closed, engineVersion, runtime: 'localNodeWorker',
    operations: [...OPERATIONS], maxInputBytes, maxPendingRequests, inFlightRequests,
    ...(reason ? { reason } : {}) });
  // Loading an addon alone is not proof of a usable engine. Probe a genuine call.
  const ready = bridge.asyncCalculate({ operation: 'query', group: 'geometry', action: 'normalize', longitude: 0 })
    .then(result => {
      engineVersion = result.engineVersion ?? null;
      available = result.errors?.length === 0 && result.data?.longitude === 0;
      if (!available) reason = 'The installed engine does not support the required query API.';
      return snapshot();
    }).catch(() => {
      reason = 'The local native SDK could not be loaded. Install its package or set ASTRO_ENGINE_MODULE.';
      return snapshot();
    });

  async function handle(req, res) {
    let pathname;
    try { pathname = new URL(req.url, 'http://127.0.0.1').pathname; }
    catch { return false; }
    if (!pathname.startsWith('/api/astro/')) return false;
    let reserved = false;
    try {
      checkLocalRequest(req);
      if (pathname === '/api/astro/status') {
        if (req.method !== 'GET') {
          sendJson(res, 405, { error: { code: 'METHOD_NOT_ALLOWED', message: 'Use GET for runtime status.' } }, { Allow: 'GET' });
          req.resume(); return true;
        }
        await ready;
        sendJson(res, available && !closed ? 200 : 503, snapshot());
        return true;
      }
      if (pathname !== '/api/astro/calculate') throw new HostError(404, 'UNKNOWN_ROUTE', 'Unknown local runtime route.');
      if (req.method !== 'POST') {
        sendJson(res, 405, { error: { code: 'METHOD_NOT_ALLOWED', message: 'Use POST for engine requests.' } }, { Allow: 'POST' });
        req.resume(); return true;
      }
      if (req.headers['content-type']?.split(';')[0].trim().toLowerCase() !== 'application/json')
        throw new HostError(415, 'JSON_REQUIRED', 'Send the raw engine request as application/json.');
      if (inFlightRequests >= maxPendingRequests)
        throw new HostError(429, 'RUNTIME_BUSY', 'The local runtime queue is full. Retry when a calculation finishes.');
      inFlightRequests++; reserved = true;
      await ready;
      if (!available || closed) throw new HostError(503, 'ENGINE_UNAVAILABLE', reason || 'The local runtime is closed.');
      const raw = await readRawBody(req, maxInputBytes);
      // Do not JSON.parse/re-serialize: preserve exact integer literals for Rust.
      const result = await bridge.asyncCalculate(raw);
      sendJson(res, 200, result);
      return true;
    } catch (error) {
      req.resume();
      if (!(error instanceof HostError)) {
        available = false; reason = 'The local native worker is unavailable. Restart the local host.';
      }
      sendJson(res, error.status || 503, { error: { code: error.code || 'ENGINE_UNAVAILABLE',
        message: error instanceof HostError ? error.message : reason } });
      return true;
    } finally { if (reserved) inFlightRequests--; }
  }
  async function close() {
    closed = true; available = false; reason = 'The local runtime is closed.';
    await bridge.close();
  }
  return Object.freeze({ handle, close, ready });
}
module.exports = { createLocalEngineHost, resolveEngineModule, OPERATIONS, MAX_INPUT_BYTES };
