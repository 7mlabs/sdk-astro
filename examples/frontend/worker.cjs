'use strict';
const { parentPort, workerData } = require('node:worker_threads');
const { calculateJson } = require(workerData.engineModule || '@7mlabs/astrology');

// One persistent worker processes native synchronous calculations sequentially.
parentPort.on('message', ({ id, requestJson }) => {
  try {
    parentPort.postMessage({ id, result: JSON.parse(calculateJson(requestJson)) });
  } catch (error) {
    parentPort.postMessage({ id, error: { name: error.name, message: error.message } });
  }
});
