#[cfg(feature = "boa")]
pub mod boa;
#[cfg(feature = "quickjs")]
pub mod quickjs;

const PORTABLE_MATH: &str = r#"
(() => {
    const call = globalThis.__plugin_math;
    delete globalThis.__plugin_math;
    for (const [code, name] of ['sin', 'cos', 'tan', 'exp', 'log', 'sqrt'].entries()) {
        Math[name] = x => call(code, +x, 0);
    }
    Math.atan2 = (y, x) => call(6, +y, +x);
    Math.pow = (x, y) => call(7, +x, +y);
    Math.hypot = (...args) => {
        const xs = args.map(x => Math.abs(+x));
        const max = Math.max(0, ...xs);
        if (xs.some(x => x === Infinity)) return Infinity;
        if (Number.isNaN(max) || max === 0) return max;
        if (xs.length === 2) return call(8, xs[0], xs[1]);
        return max * Math.sqrt(xs.reduce((sum, x) => sum + (x / max) * (x / max), 0));
    };
})();
"#;

pub fn source(text: &[u8]) -> String {
    let text = std::str::from_utf8(text).unwrap();
    let math = if text.starts_with("// Built-in math\n") {
        ""
    } else {
        PORTABLE_MATH
    };
    format!(
        "{}\ndelete globalThis.Date; delete Math.random;\n{math}\n{text}",
        plugin_spike_wire::js_layout()
    )
}

#[cfg(any(feature = "boa", feature = "quickjs"))]
fn math(code: i32, x: f64, y: f64) -> f64 {
    let output = match code {
        0 => libm::sin(x),
        1 => libm::cos(x),
        2 => libm::tan(x),
        3 => libm::exp(x),
        4 => libm::log(x),
        5 => libm::sqrt(x),
        6 => libm::atan2(x, y),
        7 if x.abs() == 1.0 && y.is_infinite() => f64::NAN,
        7 => libm::pow(x, y),
        8 => libm::hypot(x, y),
        _ => unreachable!(),
    };
    if output.is_nan() { f64::NAN } else { output }
}
