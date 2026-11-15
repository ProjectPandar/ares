# STL format detection and binary record sizing

## Source and Rust boundary

Authority: OrcaSlicer 2.5.0,
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

Port `deps_src/admesh/stlinit.cpp:62–104`,
`stl_open_count_facets`, into `crates/ares-core/src/stl.rs`:
format selection in `looks_like_binary` / `load`, and record sizing in
`parse_binary`. Constants come from `deps_src/admesh/stl.h:34–39,73`:
80-byte standard header, 4-byte facet-count field, 50-byte facet record,
and minimum binary file size284.

This replaces the existing exact-header-count-only detector with the cited
admesh behavior. It is not a new Ares import policy or a compatibility mode.
The core remains a platform-neutral byte-input API.

## Current differences

- Ares requires `84 + header_count * 50 == input.len()` to select binary.
  Admesh first checks128 body bytes starting at offset84 for a byte above127
  (`stlinit.cpp:62–79`). Such a byte selects binary even if the declared count
  is wrong.
- If that body probe looks like ASCII, admesh checks the declared count
  against the complete file length using `uint64_t` (`:80–88`). Zero normals
  and coordinates such as10 or15 can otherwise make binary records appear
  entirely ASCII-like.
- Admesh requires the complete128-byte probe. A shorter input is rejected,
  rather than allowing a partial probe to affect selection (`:65–69`).
- Once binary is selected, admesh checks minimum size and record alignment
  (`:99–102`) and derives the facet count from file size (`:104`). Ares instead
  trusts the declared count and does not apply the same minimum-size check.
- Ares's saturating `usize` multiplication does not protect the subsequent
  addition. On wasm32, ASCII bytes interpreted as a count can saturate the
  product and overflow the header addition.

## Required behavior

1. Reject inputs shorter than the complete body probe at the STL loader.
   The boolean sniffer must remain safe for shorter unknown-format inputs.
2. Select binary when any byte in that probe is above127.
3. Otherwise perform the header-count/file-length equality in `u64`.
   Read the count as little-endian `u32`; do not use saturation or wrapping.
4. For binary input, require size at least284 and an integral number of
   50-byte records after the84-byte prefix, before allocating or reading facets.
5. Derive the number of parsed records from the actual byte length. An
   incorrect header count is not a rejection when the body probe selects
   binary and the actual record layout is valid.
6. Preserve existing ASCII grammar, finite-coordinate checks, and facet repair.
   Do not change import geometry to compensate for classification differences.

The largest header-derived length is
`84 + (2^32 - 1) * 50 = 214748364834`, below `2^38`. Multiplication and
addition in `u64` are therefore exact on every supported target. Input length
also fits `u64` on browser WASM, Windows, macOS, and Linux. Binary parser
allocation and offsets are bounded by the validated actual input length,
not by untrusted header arithmetic.

## Included and deferred behavior

Included: the complete body probe, ASCII-to-binary length fallback,
minimum/aligned binary sizing, and actual-size facet count for the standard
80-byte STL header.

Deferred: custom header-length APIs, filesystem opening/seeking, host-endian
file-reading machinery, terminal warnings for header-count mismatches
(`stlinit.cpp:106–118`), adjacent ASCII grammar changes, and further mesh repair.
Little-endian byte decoding already replaces host-endian conversion portably.
No legacy detector, target-specific branch, extra format option, or fallback
around the old arithmetic remains.

## Public E2E contract

Use the public `ares_core::load_model` API and OrcaSlicer's CLI STL importer /
STL exporter. Generate every input at runtime from synthetic geometry. Compare
admission for rejected inputs and exact ordered facet vertices for accepted
inputs, without sorting, coordinate masks, or authored Native output files.
Save input bytes, commands, statuses, Native exports, and comparison JSON as
repeatable artifacts.

Failure cases and controls:

- Binary tetrahedra with a high-bit body probe and a declared count of zero,
  too few facets, or `u32::MAX`: actual-size parsing must match Native.
- Valid UTF-8 ASCII STL with a high-bit solid-name byte inside the probe:
  reproduce Native's binary classification and malformed-size rejection.
- Ordinary synthetic ASCII STL whose bytes80–83 imply a length above the
  wasm32 address space: preserve successful ASCII import and exact geometry.
- Binary records using only bytes below128 in the probe: exercise the
  header-length fallback, including a header beginning with `solid`.
- High-bit binary inputs below the minimum size or with incomplete records:
  reject as Native does; also retain the complete-probe short-input rule.

No extra checked-wasm build, browser harness, framework, or dependency is
introduced. A native64-bit test cannot reproduce the arithmetic panic itself,
but these native E2E cases expose the independently observable classification
and record-sizing differences on every platform. Checked wasm32 builds trap
on the old overflow; unchecked release builds can wrap instead, so a test that
requires a trap would depend on build settings rather than STL semantics.
The portable behavior checks plus the bounded `u64` arithmetic remove that
failure structurally without relying on a particular overflow-check profile.
