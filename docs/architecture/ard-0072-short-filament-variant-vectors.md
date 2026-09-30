# ARD-0072: Short filament variant payloads

Upstream 8500fcdcc boundary: `PrintConfig.cpp:9806-10022`,
`DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments`.
Each output vector is value-initialized to the filament-map count. A selected
variant outside a payload vector leaves a zero slot (false, empty string,
numeric zero, enum discriminant zero); nullable numeric zero is not nil.
This differs from `update_values_to_printer_extruders` at 9693-9803, which
uses `get_at` and may broadcast the first printer/process value.

Rust boundary: `options/project_variants/filament.rs` and a small separate
selector module. Preserve printer/process selector semantics and raw source
immutability. Existing typed materialization is a temporary compatibility
shell around DynamicPrintConfig, not an independently designed pipeline.
Include the 37 already mapped filament variant fields and their actual
scalar storage types. Defer auto filament-map derivation and the ARD-0071
statistics footer. Empty payload vectors retain the inherited invalid-input
restriction and are not claimed as parity; an empty string element is distinct
from an empty vector. No source-token pinning or output-filter changes.

Public seam: native-generated project bytes to complete G-code. Use a valid
two-colour H2D cube, explicit manual map, zero-density boundary to omit the
separately deferred weight/cost lines, and disabled M73/prime tower. Retain
complete original failing fixtures as separate observations, not filtered
comparison inputs. Require complete RED/GREEN and reversion sensitivity.
