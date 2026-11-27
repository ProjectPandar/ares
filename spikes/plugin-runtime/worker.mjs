import * as browser from './webassembly.mjs';
import * as storage from './file-storage.mjs';
import { HEADER, METADATA, PATH_COUNT, LAYER_INDEX, Z_BOTTOM, LAYER_HEIGHT, LAYERS, TRIALS } from './layout.mjs';

async function bytes(name) {
    const response = await fetch(name);
    if (!response.ok) throw new Error(`fetch ${name}: ${response.status}`);
    return new Uint8Array(await response.arrayBuffer());
}

function setLayer(data, layer) {
    const view = new DataView(data.buffer);
    view.setUint32(LAYER_INDEX, layer, true);
    view.setFloat64(Z_BOTTOM, layer * LAYER_HEIGHT, true);
    const paths = view.getUint32(PATH_COUNT, true);
    for (let path = 0; path < paths; path++) {
        const inset = (path + layer) % 4;
        const pos = HEADER + path * METADATA;
        view.setUint32(pos, inset, true);
        view.setUint32(pos + 4, Number(inset !== 0), true);
    }
}

function same(actual, expected) {
    if (actual.length !== expected.length || !actual.every((byte, i) => byte === expected[i])) {
        throw new Error('unexpected output bytes');
    }
}

async function sha256(bytes) {
    const hash = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
    return Array.from(hash, byte => byte.toString(16).padStart(2, '0')).join('');
}

async function metering() {
    const { default: init, instrumentPlugin } = await import('./metering_host.js');
    await init({ module_or_path: './metering_host_bg.wasm' });
    return instrumentPlugin;
}

async function run({ backend, engine, kind, rawMath, metered = false, portable = false, runnerReload = false, layers = LAYERS, trials = TRIALS }) {
    const js = engine !== 'rust';
    const direct = backend === 'direct';
    const compiled = direct && !js;
    const [input, original, reloaded, gcodeInput, wasm, replacement, source] = await Promise.all([
        bytes('layer.bin'), bytes(`${kind}-original.bin`), bytes(`${kind}-reloaded.bin`), bytes('gcode.bin'),
        direct ? undefined : bytes(`${js ? engine : kind}.wasm`),
        compiled ? undefined : bytes(`${kind}-alternate.${js ? 'js' : 'wasm'}`),
        js ? bytes(rawMath ? 'raw-math.js' : `${kind}.js`) : undefined,
    ]);
    let load = browser.load;
    let interpreter;
    if (portable) {
        const instrument = await metering();
        load = (module, source) => browser.load(instrument(module, 1024), source, 1_000_000_000n);
    } else if (backend === 'interpreter') {
        const { default: init, SpikeInterpreter } = await import('./ares_wasm.js');
        await init({ module_or_path: './ares_wasm_bg.wasm' });
        interpreter = new SpikeInterpreter(metered);
        load = interpreter.load.bind(interpreter);
    } else if (direct) {
        const { default: init, SpikeDirect } = await import(`./direct_${engine}.js`);
        await init({ module_or_path: `./direct_${engine}_bg.wasm` });
        load = (_, source) => new SpikeDirect(engine, kind === 'subdivision', source ?? new Uint8Array());
    }
    const samples = [];
    for (let trial = 0; trial < trials; trial++) {
        let t = performance.now();
        let guest = load(wasm, source);
        const load_ms = performance.now() - t;
        setLayer(input, 0);
        t = performance.now();
        const first = guest.perimeters(input);
        const warmup_ms = performance.now() - t;
        same(first, original);
        const output = new Uint8Array(layers * first.length);
        let hook_total_ms = 0;
        const wall = performance.now();
        for (let layer = 0; layer < layers; layer++) {
            setLayer(input, layer);
            t = performance.now();
            const layerEdits = guest.perimeters(input);
            hook_total_ms += performance.now() - t;
            output.set(layerEdits, layer * layerEdits.length);
        }
        const wall_ms = performance.now() - wall;
        const gcode = guest.gcode(gcodeInput);
        same(gcode.subarray(0, gcodeInput.length), gcodeInput);
        setLayer(input, 0);
        let reload_ms = null;
        if (!compiled) {
            t = performance.now();
            if (js) {
                guest.set_source(replacement);
            } else {
                const next = load(replacement);
                guest.free();
                guest = next;
            }
            const updated = guest.perimeters(input);
            reload_ms = performance.now() - t;
            same(updated, reloaded);
        }
        guest.free();
        let runner_reload_ms = null;
        if (js && runnerReload) {
            t = performance.now();
            const next = load(wasm, replacement);
            const updated = next.perimeters(input);
            runner_reload_ms = performance.now() - t;
            same(updated, reloaded);
            next.free();
        }
        samples.push({ load_ms, warmup_ms, hook_total_ms, wall_ms, reload_ms, runner_reload_ms,
            layer_sha256: await sha256(output), gcode_sha256: await sha256(gcode),
            gcode: new TextDecoder().decode(gcode) });
    }
    interpreter?.free();
    return samples;
}

async function component({ jco }) {
    const wasm = await bytes('component.wasm');
    const replacement = await bytes('component-alternate.wasm');
    let direct;
    try {
        await WebAssembly.compile(wasm);
        direct = 'accepted';
    } catch (error) {
        direct = String(error);
    }
    if (!jco) return { direct };
    const boot = performance.now();
    const { generate } = await import('/jco/dist/runtime.js');
    const import_ms = performance.now() - boot;
    async function use(wasmBytes, expected) {
        const t = performance.now();
        const { files } = await generate(wasmBytes, {
            name: 'plugin', noTypescript: true, instantiation: { tag: 'async' }, noNodejsCompat: true,
        });
        const output = new Map(files);
        const url = URL.createObjectURL(new Blob([output.get('plugin.js')], { type: 'text/javascript' }));
        const { instantiate } = await import(url);
        URL.revokeObjectURL(url);
        const guest = await instantiate(path => WebAssembly.compile(output.get(path)), {});
        same(guest.perimeters(new Uint8Array([0, 1, 2])), new Uint8Array([expected, 1, 2]));
        return performance.now() - t;
    }
    const first_ms = await use(wasm, 7);
    const reload_ms = await use(replacement, 9);
    const resources = performance.getEntriesByType('resource').filter(e => new URL(e.name).pathname.startsWith('/jco/'));
    return { direct, import_ms, first_ms, reload_ms,
        resources: resources.map(e => ({ file: new URL(e.name).pathname, bytes: e.decodedBodySize })) };
}

async function limits() {
    const { default: init, SpikeInterpreter } = await import('./ares_wasm.js');
    await init({ module_or_path: './ares_wasm_bg.wasm' });
    const interpreted = Array.from(SpikeInterpreter.probe(await bytes('limits.wasm')));
    const memory = new WebAssembly.Memory({ initial: 1, maximum: 2 });
    const module = new WebAssembly.Module(await bytes('limits-browser.wasm'));
    const { exports } = new WebAssembly.Instance(module, { ares: { memory } });
    exports.advance(100000);
    const native = [0, new DataView(memory.buffer).getInt32(0, true), exports.grow(2)];
    const results = { interpreter: interpreted, browser: native };
    for (const engine of ['quickjs', 'boa']) {
        const { default: init, SpikeDirect } = await import(`./direct_${engine}.js`);
        await init({ module_or_path: `./direct_${engine}_bg.wasm` });
        results[engine] = Array.from(SpikeDirect.probe(engine));
    }
    return results;
}

async function portableLimits() {
    const instrument = await metering();
    const input = await bytes('limits.wasm');
    const transformed = instrument(input, 2);
    const exports = browser.instantiate(transformed);
    let trapped = false;
    exports.__ares_fuel.value = 1000n;
    try {
        exports.advance(100000);
    } catch (error) {
        if (!(error instanceof WebAssembly.RuntimeError)) throw error;
        trapped = true;
    }
    const steps = new DataView(exports.memory.buffer).getInt32(0, true);
    exports.__ares_fuel.value = 1000n;
    const atCap = exports.grow(1);
    const aboveCap = exports.grow(1);
    if (!trapped || steps !== 71 || atCap !== 1 || aboveCap !== -1) {
        throw new Error(`expected trap at 71 and two-page cap; got ${[trapped, steps, atCap, aboveCap]}`);
    }
    return { limit: [Number(trapped), steps, atCap, aboveCap], sha256: await sha256(transformed) };
}

async function measure(scenario) {
    if (scenario.portableLimits) return portableLimits();
    if (scenario.component) return component(scenario);
    if (scenario.limits) return limits();
    if (scenario.files) return storage.measure(await bytes(`files-${scenario.files}.wasm`), scenario.files, TRIALS);
    return run(scenario);
}

self.onmessage = async ({ data }) => {
    try {
        self.postMessage({ result: await measure(data) });
    } catch (error) {
        self.postMessage({ error: error.stack ?? String(error) });
    }
};
