// Web Worker: runs PRP bucket queries independently.
// Each worker loads its own WASM instance.

let wasm = null;

self.onmessage = async function(e) {
  const { type, payload } = e.data;

  if (type === 'init') {
    // Load WASM module from the URL provided by main thread.
    const { wasmUrl } = payload;
    try {
      const mod = await import(wasmUrl);
      await mod.default();
      wasm = mod;
      self.postMessage({ type: 'ready' });
    } catch (err) {
      self.postMessage({ type: 'error', error: err.toString() });
    }
    return;
  }

  if (type === 'run') {
    const { prp, domain, rounds, t, bucketStart, bucketCount } = payload;
    try {
      let totalMs = 0;
      if (prp === 'hoang') {
        totalMs = wasm.bench_hoang_n_buckets(domain, rounds, t, bucketCount);
      } else if (prp === 'alf') {
        totalMs = wasm.bench_alf_n_buckets(domain, t, bucketCount);
      } else if (prp === 'fastprp') {
        // Run one_bucket in a loop since n_buckets may not be available
        let total = 0;
        for (let i = 0; i < bucketCount; i++) {
          const r = wasm.bench_fastprp_one_bucket(domain, t);
          total += r[0] + r[1]; // build + query
        }
        totalMs = total;
      }
      self.postMessage({ type: 'done', totalMs, bucketCount });
    } catch (err) {
      self.postMessage({ type: 'error', error: err.toString() });
    }
    return;
  }
};
