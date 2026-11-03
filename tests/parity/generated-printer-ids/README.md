# Generated printer-id public-input witness

Source: OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PrintConfig.cpp:9004–9060::get_index_for_extruder` and variant-update calls
starting at9634. Spec:703658c84, `docs/architecture/generated-printer-id-materialization.md`.

`input.3mf` is preserved byte-for-byte from
`${TMP_ROOT}/ares-x2d-flush-matrix/green/generated-ids-legal-red/case-75aoF4/input.3mf`,
SHA256 `600d0bdd7b626855411a6d85b342fc1fc0a0b85b6a676e998a742c429d7e9d08`.
It is a constructed stock X2D/cube10 CLI fixture, not customer/GUI evidence.
Historical test provenance:b39047a8884db752dbd702a4c67cbca763fc2cb3.

Two nozzles have explicit Standard types and Direct Drive/Bowden extruder
selectors. Four ordered variants share two physical variant groups; the
printer-id vector contains only1,2 and cannot cover four variants. Matrix is
absent. No scalar nozzle UB witness or numerical expected-value substitution.

Confirmed seam: public project bytes through slice_project (existing complete
artifact comparator) versus fresh identified Native CLI on identical bytes.
Failure inventory before production: initial generated-id lookup, metadata
mutation during variant-one, reselection during variant-two, short id vector
get_at semantics, deferred flush dependency after selection, and complete
output drift. Preserve each RED and stop for a boundary revision if the witness
requires adjacent FM behavior or an undefined source read. No private-method
assertions, success assertion on an Ares-derived snapshot, mock, ignore, or
relaxed comparison.

Repeat with provisioned v2 seeded oracle and native producer/resources identity:
`cargo nextest run -p ares-cli --test orca_parity -E 'test(generated_printer_ids)'`.
Artifact root and Native cache root must be fresh and explicitly recorded.
The test emits a verifiable input/reference/Ares-or-error manifest through the
existing ordered-byte comparator and embeds producer/request provenance.
