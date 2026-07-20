# Task 22M Package 5 MSRV Signed-Zero Amendment Plan

## Contract

This plan implements only the signed-zero portability repair in the companion
amendment specification. It supplements the approved Task 22M specification
and plan (SHA-256
`5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff` /
`b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`)
and the approved Package 5 coverage-repair specification and plan (SHA-256
`f7a79e4a80fad9d91609e8934d7260bea62016a9076a00c86926fa216b790c17` /
`a19bbfd753690bf3fdb8696f95b25d5ba0c74d82c91e7a2f8bc2bc1097ea65cf`).

Fixed identities remain OrcaSlicer commit/tree
`8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549` and Ares baseline commit/tree
`fcd2c5728f4c0529f28bfc43c636507d61e263d8` /
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

## Allowed Paths

- modify `crates/ares-core/src/project_slice/elephant_foot/profile.rs`;
- modify `crates/ares-core/src/project_slice/tests/elephant_foot/profile.rs`.

The final tracked manifest is exactly 64 paths: the approved exact 62-path
frame plus this specification and plan. No other source, test, Cargo, adapter,
workflow, fixture, or documentation path is authorized.

## Steps

1. Freeze the isolated Rust 1.91.0 RED for the existing strict-equality test
   and the isolated Rust 1.96.1 GREEN. Record the differing bits at accumulated
   indices 1 and 8.
2. Obtain independent fixed-source/specification and current-Ares/plan approval
   before modifying either Rust file.
3. Replace `f32::max` only at the fixed-source Laplacian selection with an
   explicit `<` branch that returns `current[index]` only when the Laplacian is
   smaller and otherwise returns the Laplacian first operand.
4. Change only accumulated expected indices 1 and 8 to positive-zero bits.
   Keep the same test name and every other expected bit.
5. Run the focused test with isolated Rust 1.91.0 and 1.96.1 target directories,
   then run exactly 81 Task 22M tests under both toolchains. Reconfirm exact
   synthetic and KSR M identities.
6. Run Task 22L, strict all-target/all-feature core clippy on Rust 1.91.0,
   core all-feature WASM, rustfmt, LOC, macro/unsafe, and diff gates.
7. Freeze the two post-repair Rust hashes and return all evidence to the same
   read-only reviewer. Repair and revalidate until P0-P3 are empty.

## Gate

This is a source-parity fix, not a relaxed test. Rust 1.91.0 must go from RED
to GREEN while Rust 1.96.1, the 81-test count, and every released M identity
remain GREEN. Any wider change blocks implementation pending another amendment.
