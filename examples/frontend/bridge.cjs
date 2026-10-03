'use strict';
const path = require('node:path');
const { Worker } = require('node:worker_threads');

function createEngineBridge(options = {}) {
  const worker = new Worker(path.join(__dirname, 'worker.cjs'), {
    workerData: { engineModule: options.engineModule || process.env.ASTRO_ENGINE_MODULE || '@7mlabs/astrology' }
  });
  const pending = new Map();
  let nextId = 1;
  let closed = false;
  let closePromise;

  function rejectPending(error) {
    for (const request of pending.values()) request.reject(error);
    pending.clear();
  }
  worker.on('message', message => {
    const request = pending.get(message.id);
    if (!request) return;
    pending.delete(message.id);
    if (message.error) {
      const error = new Error(message.error.message);
      error.name = message.error.name;
      request.reject(error);
    } else request.resolve(message.result);
  });
  worker.on('error', error => { closed = true; rejectPending(error); });
  worker.on('exit', code => {
    closed = true;
    rejectPending(new Error(`Astrology worker exited with code ${code}`));
  });

  function asyncCalculate(input) {
    if (closed) return Promise.reject(new Error('Astrology bridge is closed'));
    let requestJson;
    try {
      requestJson = typeof input === 'string' ? input : JSON.stringify(input);
      if (typeof requestJson !== 'string') throw new TypeError('Expected a serializable engine request or raw JSON string');
    } catch (error) { return Promise.reject(error); }
    const id = nextId++;
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
      try { worker.postMessage({ id, requestJson }); }
      catch (error) { pending.delete(id); reject(error); }
    });
  }

  function close() {
    if (!closePromise) {
      closed = true;
      rejectPending(new Error('Astrology bridge is closed'));
      closePromise = worker.terminate();
    }
    return closePromise;
  }
  return Object.freeze({ asyncCalculate, close });
}
module.exports = { createEngineBridge };
