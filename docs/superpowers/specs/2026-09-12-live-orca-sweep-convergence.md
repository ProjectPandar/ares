
## Qidi X-Max 3 CLOSED — template scalars type as integers (2026-09-20 xl)

The end-gcode template `G1 Z{max_print_height / 2 + 10} F600`:
upstream's placeholder engine types integer-looking scalars
`TYPE_INT` and its `/` between ints is INTEGER division
(`PlaceholderParser.cpp:496-503`: `m_data.i /= rhs.i()`) —
`printable_height = 315` → `315/2 = 157` → Z167. The ares parsed all
config-block scalars as f64 (`Value::Number`), giving 157.5 → Z167.5.

Fix (`value.rs::parse_scalar`): try `i64` before `f64` — integral
scalars become `Value::Integer` (the arithmetic layer already
int-divides Integer operands; `format_number` prints integral floats
without decimals, so formatting is unchanged). Qidi X-Max 3 0.2
byte-identical 4/4 fresh oracle runs; 0.4 3/4; 0.6 4/4; 0.8 2/4 (the
misses are oracle race variants). MK3S (r6-r8 byte-exact), E3V3+,
CORE One L, XL 0.4 unchanged; ares-core 6993/6993.
