function open() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open('plugin-spike-files', 1);
        request.onupgradeneeded = () => request.result.createObjectStore('plugin-data');
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
    });
}

function transaction(db, write, fn) {
    return new Promise((resolve, reject) => {
        const tx = db.transaction('plugin-data', write ? 'readwrite' : 'readonly');
        const request = fn(tx.objectStore('plugin-data'));
        tx.oncomplete = () => resolve(request.result);
        tx.onabort = tx.onerror = () => reject(tx.error);
    });
}

export async function measure(wasm, mode, trials) {
    const { default: init, SpikeFiles } = await import('./files_host.js');
    await init({ module_or_path: './files_host_bg.wasm' });
    const db = await open();
    const key = `sample/${mode}/result.txt`;
    const old = new TextEncoder().encode('slice-start bytes');
    try {
        await transaction(db, true, store => store.put(old, key));
        let t = performance.now();
        const prior = await transaction(db, false, store => store.get(key));
        const snapshot_ms = performance.now() - t;
        const runs = [];
        let staged;
        for (let trial = 0; trial < trials; trial++) {
            t = performance.now();
            const probe = new SpikeFiles(wasm, mode === 'wasi', prior);
            const load_ms = performance.now() - t;
            try {
                probe.run(1);
                t = performance.now();
                probe.run(10_000);
                runs.push({ load_ms, run_ms: performance.now() - t, iterations: 10_000 });
                staged = probe.finish(prior);
            } finally {
                probe.free();
            }
        }
        t = performance.now();
        await transaction(db, true, store => store.put(staged, key));
        const commit_ms = performance.now() - t;
        const persisted = await transaction(db, false, store => store.get(key));
        if (new TextDecoder().decode(persisted) !== 'virtual root specimen') {
            throw new Error('committed data did not match the package');
        }
        return { runs, snapshot_ms, commit_ms };
    } finally {
        db.close();
    }
}
