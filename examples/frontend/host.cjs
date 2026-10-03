'use strict';
const { createServer } = require('node:http');
const { readFile, realpath, stat } = require('node:fs/promises');
const path = require('node:path');
const { createLocalEngineHost } = require('./local-http.cjs');

const types = { '.html': 'text/html; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.svg': 'image/svg+xml',
  '.woff2': 'font/woff2' };
function createStandaloneServer(options = {}) {
  const engine = createLocalEngineHost(options);
  const uiRoot = path.resolve(__dirname, 'ui');
  const server = createServer(async (req, res) => {
    if (await engine.handle(req, res)) return;
    try {
      if (!['GET', 'HEAD'].includes(req.method)) { res.writeHead(405, { Allow: 'GET, HEAD' }); return res.end(); }
      const pathname = decodeURIComponent(new URL(req.url, 'http://127.0.0.1').pathname);
      const relative = pathname === '/' ? 'example.html' : pathname.startsWith('/ui/') ? pathname.slice(4) : null;
      if (relative === null) throw new Error('Not found');
      const filename = path.resolve(uiRoot, relative);
      if (!filename.startsWith(uiRoot + path.sep) || !types[path.extname(filename)]) throw new Error('Not found');
      const actual = await realpath(filename);
      if (!actual.startsWith(uiRoot + path.sep) || !(await stat(actual)).isFile()) throw new Error('Not found');
      const data = await readFile(actual);
      res.writeHead(200, { 'Content-Type': types[path.extname(actual)], 'Cache-Control': 'no-store',
        'X-Content-Type-Options': 'nosniff', 'Referrer-Policy': 'same-origin',
        'Content-Security-Policy': "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; object-src 'none'; base-uri 'none'; frame-ancestors 'none'" });
      res.end(req.method === 'HEAD' ? undefined : data);
    } catch { res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' }); res.end('Not found'); }
  });
  server.requestTimeout = 30_000;
  server.headersTimeout = 15_000;
  let started = false;
  async function start(port = Number(process.env.PORT || 4174)) {
    if (!Number.isInteger(port) || port < 0 || port > 65535) throw new RangeError('PORT must be an integer0..65535');
    if (started) throw new Error('The local host is already started');
    started = true;
    try {
      await new Promise((resolve, reject) => {
        const failed = error => reject(error);
        server.once('error', failed);
        server.listen(port, '127.0.0.1', () => { server.off('error', failed); resolve(); });
      });
    } catch (error) { await engine.close(); throw error; }
    return `http://127.0.0.1:${server.address().port}`;
  }
  async function close() {
    await engine.close();
    if (server.listening) {
      const closed = new Promise(resolve => server.close(resolve));
      server.closeAllConnections?.();
      await closed;
    }
  }
  return Object.freeze({ server, engine, start, close });
}
module.exports = { createStandaloneServer };
if (require.main === module) {
  const host = createStandaloneServer();
  host.start().then(async url => {
    console.log(`Astro local UI: ${url}`);
    const status = await host.engine.ready;
    console.log(status.available ? `Native engine ${status.engineVersion}, local worker` : status.reason);
  }).catch(async error => { console.error(error.message); await host.close(); process.exitCode = 1; });
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, async () => { await host.close(); process.exit(0); });
}
