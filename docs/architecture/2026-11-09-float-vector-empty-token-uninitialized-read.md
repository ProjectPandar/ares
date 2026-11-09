# ARD: Float-vector empty tokens read an uninitialized upstream double

Date: 2026-11-09
Scope: empty comma fields after trimming in a nonempty `coFloats` wire.

## Upstream boundary

At OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/Config.hpp:932–958`,
`ConfigOptionFloatsTempl<NULLABLE>::deserialize` clears the vector unless
append is set, splits with comma-separated `std::getline`, trims each field,
and handles `nil` before numeric extraction.

At `Config.hpp:951–954`, the numeric branch constructs an
`std::istringstream` from the field, declares `double value;` without
initialization, extracts into it, and appends it without checking the
stream failure flag. If trimming leaves an empty field, the formatted
input sentry encounters an empty stream before numeric extraction can
write to `value`. Reading that uninitialized double for the vector is undefined
behavior.

## Decision

Ares does not reproduce this uninitialized-memory read or assign a numeric
value to an empty field. The typed nullable-float loading path keeps its
invalid-float error for that input. Its Rust boundary is
`options/model_config_deserialize/wire.rs` together with
`options/config_types/nullable_vector.rs` and the raw project input boundary in
`options/json_deserialize.rs`. Both use the same joined-wire decoder before
numeric validation and export.

This case does not make nonnumeric nonempty fields undefined. For those
fields, C++11 and later numeric extraction writes zero on conversion
failure. The vector appends that zero because it does not test the
failure flag. Ares implements that defined behavior.

A numeric prefix is accepted and suffix text does not turn the value into
a relative percentage. The `nil` branch remains separate: nullable
vectors append their nil sentinel, while non-nullable vectors throw.
An entirely empty wire uses the explicit empty-wire branch at
`Config.hpp:937–939`, producing one zero value. A final comma alone does
not create another field through `std::getline`.

## Fixture rule

Native parity fixtures must avoid fields that become empty after trimming
inside a nonempty float-vector wire. This includes empty interior fields
and whitespace-only fields. Such inputs cannot establish a Native value
contract. Rust-only rejection checks may exercise this boundary without
executing the upstream uninitialized read.
