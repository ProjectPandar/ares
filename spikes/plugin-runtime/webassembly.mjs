const unavailable = () => 52;
const wasi = {
    random_get: unavailable,
    fd_write: unavailable,
    environ_get: unavailable,
    environ_sizes_get: unavailable,
    poll_oneoff: unavailable,
    sched_yield: unavailable,
    proc_exit(code) { throw new Error(`guest proc_exit: ${code}`); },
};

export function instantiate(moduleBytes) {
    const compiled = new WebAssembly.Module(moduleBytes);
    let guest;
    ({ exports: guest } = new WebAssembly.Instance(compiled, {
        wasi_snapshot_preview1: { ...wasi, clock_time_get(_clock, _precision, output) {
            if (!guest || output < 0 || output + 8 > guest.memory.buffer.byteLength) return 21;
            new DataView(guest.memory.buffer).setBigUint64(output, 0n, true);
            return 0;
        } },
        env: { __rquickjs_host_now_us: () => 0 },
    }));
    return guest;
}

export function load(moduleBytes, source, fuel) {
    const guest = instantiate(moduleBytes);
    function write(input) {
        if (fuel !== undefined) guest.__ares_fuel.value = fuel;
        const pos = guest.reserve_input(input.length);
        new Uint8Array(guest.memory.buffer, pos, input.length).set(input);
    }
    function hook(call, input) {
        write(input);
        const packed = call();
        const pos = Number(packed & 0xffffffffn);
        const len = Number(packed >> 32n);
        return new Uint8Array(guest.memory.buffer, pos, len).slice();
    }
    const instance = {
        set_source(bytes) { write(bytes); guest.set_source(); },
        perimeters(bytes) { return hook(guest.perimeters, bytes); },
        gcode(bytes) { return hook(guest.gcode_post_process, bytes); },
        free() {},
    };
    if (source) instance.set_source(source);
    return instance;
}
