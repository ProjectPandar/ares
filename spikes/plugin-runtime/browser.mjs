import { chromium } from '../../web/node_modules/@playwright/test/index.mjs';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const out = process.env.PLUGIN_SPIKE_OUT;
const jcoDir = process.env.PLUGIN_SPIKE_JCO_DIR;
const chrome = process.env.ARES_E2E_CHROME;
if (!out || !chrome) throw new Error('set PLUGIN_SPIKE_OUT and ARES_E2E_CHROME');
let jco;
if (jcoDir) {
    const pkg = resolve(jcoDir, 'node_modules/@bytecodealliance');
    const build = await Bun.build({
        entrypoints: [resolve(pkg, 'jco/dist/browser.js')], target: 'browser',
        external: ['node:*'],
    });
    jco = { js: await build.outputs[0].text(), vendor: resolve(pkg, 'jco-transpile/vendor') };
}

const scriptDir = new URL('./', import.meta.url);
const scripts = new Set(['worker.mjs', 'webassembly.mjs', 'file-storage.mjs']);
const pageHtml = `<!doctype html><meta charset="UTF-8"><title>Plugin runtime benchmark</title>
<script>
  window.run = scenario => new Promise((resolve, reject) => {
    const worker = new Worker('/worker.mjs', { type: 'module' });
    worker.onmessage = ({data}) => {
      worker.terminate();
      if (data.error) reject(new Error(data.error));
      else resolve(data.result);
    };
    worker.onerror = event => { worker.terminate(); reject(new Error(event.message)); };
    worker.postMessage(scenario);
  });
</script>`;
const server = Bun.serve({
    hostname: '127.0.0.1', port: Number(process.env.PLUGIN_SPIKE_PORT ?? 4291),
    fetch(request) {
        const path = new URL(request.url).pathname;
        const name = path.slice(1);
        if (!name) return new Response(pageHtml, { headers: {
            'content-type': 'text/html',
            'cross-origin-opener-policy': 'same-origin',
            'cross-origin-embedder-policy': 'require-corp',
        } });
        if (name === 'jco/dist/runtime.js' && jco) {
            return new Response(jco.js, { headers: { 'content-type': 'text/javascript' } });
        }
        let file;
        if (jco && name.startsWith('jco/vendor/')) file = resolve(jco.vendor, name.split('/').pop());
        else if (scripts.has(name)) file = new URL(name, scriptDir);
        else if (/^[a-z0-9_-]+\.(wasm|m?js|bin)$/.test(name)) file = resolve(out, name);
        if (!file) return new Response('not found', { status: 404 });
        try {
            return new Response(readFileSync(file), { headers: {
                'content-type': name.endsWith('.wasm') ? 'application/wasm' : name.endsWith('.bin') ? 'application/octet-stream' : 'text/javascript',
                'cache-control': 'no-store',
                'cross-origin-opener-policy': 'same-origin', 'cross-origin-embedder-policy': 'require-corp',
            } });
        } catch {
            return new Response('not found', { status: 404 });
        }
    },
});

const layers = Number(process.env.PLUGIN_SPIKE_LAYERS ?? 300);
const trials = Number(process.env.PLUGIN_SPIKE_TRIALS ?? 5);
let browser;
let failed = false;
try {
    browser = await chromium.launch({ executablePath: chrome, args: ['--js-flags=--wasm-num-compilation-tasks=1'] });
    console.log(`Chrome ${browser.version()}`);
    const page = await browser.newPage();
    await page.goto(server.url.href);
    async function capture(scenario, file) {
        try {
            const result = await page.evaluate(scenario => window.run(scenario), scenario);
            writeFileSync(resolve(out, file), Array.isArray(result)
                ? result.map(x => JSON.stringify(x)).join('\n') + '\n'
                : JSON.stringify(result, null, 2) + '\n');
            console.log(file);
        } catch (error) {
            failed = true;
            console.error(file, error);
        }
    }
    const combinations = [
        ['direct', 'rust'], ['direct', 'quickjs'], ['direct', 'boa'],
        ['interpreter', 'rust'], ['interpreter', 'quickjs'], ['interpreter', 'boa'],
        ['browser', 'rust'], ['browser', 'quickjs'], ['browser', 'boa'],
    ];
    const followup = process.env.PLUGIN_SPIKE_PORTABLE;
    if (followup) {
        await capture({ portableLimits: true }, 'browser-portable-limits.json');
        if (failed) throw new Error('portable WebAssembly limits did not hold');
        if (followup !== 'probe') {
            for (const kind of ['staggered', 'subdivision']) {
                for (const engine of ['rust', 'quickjs']) {
                    for (const portable of [false, true]) {
                        const label = portable ? 'portable' : 'plain';
                        await capture({ backend: 'browser', engine, kind, layers, trials, portable, runnerReload: true },
                            `browser-${label}-${engine}-${kind}.jsonl`);
                    }
                }
            }
        }
    } else {
        for (const kind of ['staggered', 'subdivision']) {
            for (const [backend, engine] of combinations) {
                await capture({ backend, engine, kind, layers, trials }, `browser-${backend}-${engine}-${kind}.jsonl`);
            }
        }
        for (const engine of ['quickjs', 'boa']) {
            for (const backend of ['browser', 'direct']) {
                await capture({ backend, engine, kind: 'staggered', rawMath: true, layers: 1, trials: 1 }, `browser-raw-math-${backend}-${engine}.jsonl`);
            }
        }
        await capture({ backend: 'interpreter', engine: 'rust', kind: 'staggered', metered: true, layers, trials }, 'browser-metered-interpreter-rust-staggered.jsonl');
        await capture({ limits: true }, 'browser-limits.json');
        for (const files of ['custom', 'wasi']) {
            await capture({ files }, 'browser-files-' + files + '.json');
        }
        await capture({ component: true, jco: Boolean(jco) }, 'component-browser.json');
    }
} finally {
    await browser?.close();
    server.stop(true);
}
if (failed) process.exitCode = 1;
