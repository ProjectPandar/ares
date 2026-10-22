# Byte semantics for placeholder regex matching

## Source boundary

This slice ports the regex comparison boundary from
`OrcaSlicer/src/libslic3r/PlaceholderParser.cpp:699-704` into
`crates/ares-core/src/project_slice/gcode_emit/expression.rs:149-151`.

The upstream implementation constructs `std::string pattern` and calls
`SLIC3R_REGEX_NAMESPACE::regex_match` with a `std::string` subject. In the
configured `std` implementation, the comparison therefore operates on the
subject's bytes: UTF-8 `é` occupies two subject bytes, and `.` matches one
byte rather than one Unicode scalar value.

## Included behavior

- Preserve whole-subject `regex_match` semantics.
- Make placeholder regex comparison use byte-oriented matching so `/./` does
  not match the two-byte UTF-8 encoding of `é`.
- Preserve the existing positive and negative regex comparison operators.

## Deferred behavior

ECMAScript regex dialect differences beyond the byte-versus-Unicode matching
boundary remain deferred, including syntax/error wording, locale behavior, and
other engine-specific differences. This slice does not change placeholder
value modeling or unrelated parser behavior.

## Verification boundary

The parity RED cases use official OrcaSlicer-exported 3MF inputs whose only
relevant difference is the machine-start-G-code comment expression:

- `"é" =~ /./` (expected to expose the byte/scalar mismatch)
- `"é" =~ /é/` (control)

Evidence and input provenance are recorded in
`/home/indexyz/ares-coord/evidence/w5-922-regex-unicode-DISCOVERY-RR.md`.
