const subdivide = false;
const oddAdjustment = 0.0;
const lowerScale = 1.0;
function resolvedOptions(view) {
    let cursor = HEADER + METADATA * view.getUint32(PATH_COUNT, true)
        + POINT_BYTES * view.getUint32(POINT_COUNT, true);
    const read32 = () => { const n = view.getUint32(cursor, true); cursor += 4; return n; };
    const text = () => {
        const length = read32();
        let out = "";
        for (let i = 0; i < length; i++) out += String.fromCharCode(view.getUint8(cursor++));
        return out;
    };
    const count = read32();
    const result = Object.create(null);
    for (let i = 0; i < count; i++) {
        const key = text();
        switch (view.getUint8(cursor++)) {
            case 0: result[key] = view.getUint8(cursor++) !== 0; break;
            case 1: result[key] = view.getFloat64(cursor, true); cursor += 8; break;
            case 2: result[key] = text(); break;
            default: throw new Error("invalid option tag");
        }
    }
    return result;
}
function perimeters(bytes) {
    const layer = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const paths = layer.getUint32(PATH_COUNT, true);
    const bottom = layer.getFloat64(Z_BOTTOM, true);
    const height = layer.getFloat64(HEIGHT, true);
    const top = bottom + height;
    const options = resolvedOptions(layer);
    const inset = path => layer.getUint32(HEADER + path * METADATA, true);
    const edits = new Uint8Array(EDIT_HEADER + paths * EDIT * 2);
    const output = new DataView(edits.buffer);
    let rank = 0;
    function emit(path, z, h) {
        const start = EDIT_HEADER + EDIT * rank++;
        output.setUint32(start, path, true);
        output.setFloat64(start + EDIT_Z, z, true);
        output.setFloat64(start + EDIT_HEIGHT, h, true);
        output.setFloat64(start + EDIT_FLOW, 1, true);
    }
    if (subdivide) {
        const lower = height * options.lower_fraction * lowerScale;
        for (let path = 0; path < paths; path++) {
            if (inset(path) === 0) emit(path, bottom + lower, lower);
        }
        for (let path = 0; path < paths; path++) {
            emit(path, top, inset(path) === 0 ? height - lower : height);
        }
    } else {
        for (let parity = 0; parity < 2; parity++) {
            for (let path = 0; path < paths; path++) {
                if (inset(path) % 2 === parity) emit(path, top + parity * (options.odd_offset + oddAdjustment), height);
            }
        }
    }
    output.setUint32(0, rank, true);
    return edits.subarray(0, EDIT_HEADER + rank * EDIT);
}

function gcodePostProcess(bytes) {
    const math = [Math.sin(0.1), Math.sin(1e20), Math.cos(-0.37), Math.tan(7),
        Math.atan2(0.83, -2.1), Math.exp(1e-9), Math.log(0.031), Math.sqrt(7),
        Math.hypot(0.17, 0.93), Math.pow(2, -0.33), Math.sin(Infinity), Math.sin(-0)];
    const bits = new DataView(new ArrayBuffer(8));
    const hex = math.map(value => {
        if (Number.isNaN(value)) return "7ff8000000000000";
        bits.setFloat64(0, value);
        return bits.getBigUint64(0).toString(16).padStart(16, "0");
    });
    const comment = "; math " + hex.join(" ") + "\n";
    const result = new Uint8Array(bytes.length + comment.length);
    result.set(bytes);
    for (let i = 0; i < comment.length; i++) result[bytes.length + i] = comment.charCodeAt(i);
    return result;
}
