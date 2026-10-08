# Complex layer-range epsilon lookup verification

Scope: source-cited PrintObjectSlice.cpp::layer_range_first (244–255),
::layer_range_next (258–266), and complex slices_to_regions (338–350, 395–398).
Fast shuffle (290–312) remains exact. MM/fuzzy, spiral ranges, adaptive/painted
profiles, active ZAA and unproved region-option families remain deferred.
See the approved epsilon lookup spec and boundary ARD.

## Independent runtime evidence

The initial legal Native half-cube wall-loop modifier produced complete Native
streams, but its no-range Core control rejected an existing unported volume
wall_loops override. This was not an epsilon RED. No guard was changed and no
unsupported rejection was made an expected test result. That attempt, and an
earlier test-only SHA256 formatting compilation error, remain disclosed.
The admitted replacement uses the already-supported sparse_infill_density
modifier (50%) overlapping half the same constant-height cube. Its no-range
whole-output control passed before production edits.

Runtime RED: exact test-only `fbc6e35dd521c4929b312ce4ddcee79f7cb9c9fd`,
run `6772faaf-30ee-430f-8eb5-93e8629d5765`, exit100. Native exited0; complete
streams differed at byte55528, line819, M73 R5 versus R6 (230653 versus231383
normalized bytes). Range start2.10005, end6, explicit height0.2, wall_loops3.
No stream segment or M73 line was normalized.

Primary GREEN: exact production `a0c219caa260060944713cc7b73ab0598ec2443b`,
run `0c81e10f-75f8-4229-82f9-c9f65421aceb`, exit0. The unchanged control and
primary witness both matched complete runtime Native streams.

The admitted release executable's incoming float zs[10] was captured with its
matching debug symbols at PrintObjectSlice.cpp:286 (mapped to289):
2.0999999046325684. The primary boundary is 0.000050095367431612914 above that
actual Z, inside EPSILON1e-4. The diagnostic exited0 and its complete output
matched the admission output after generated-by-line normalization only.
A source261 breakpoint had no compiled code; a source397 local was optimized
out. Both failed diagnostic attempts remain retained; neither is Native PASS.
Upstream assertions are release no-ops; no outcome was predicted from them.

Supplemental boundary tests were exercised against 083 plus a disclosed
**test-only overlay**, not described as the clean 083 test inventory:
run `b059a86f-4b73-4624-b49d-cfc0ced74d5d`, exit100, 2PASS/1FAIL. Four complex
exact/below/outside cases and two fast exact/near-above cases passed whole
outputs on the old production source. The exact inclusive threshold double
2.1000999046325686 had a separate runtime Native-exit0 whole-output RED.
The overlay was removed, restoring the clean baseline. No additional production
change was needed for the supplemental cases; all passed in the fresh candidate.

## Fresh acceptance pair and carry-forward

- Clean baseline `083cb1862d3d81111ef6892b96ab511a3a7d7c1f`:
  run `62a07ebe-b82e-445f-90dc-cb0e56fd9b9f`, 748PASS/0FAIL/0skip, exit0.
- Candidate `01829a3f4a23f0f364a9659a1300e830fb824577`:
  run `9f1ece86-359f-4ee8-adca-ec33f239befb`, 752PASS/0FAIL/0skip, exit0.
- Input-bound complete-output audit:611 identity groups, **628 actual common
  Ares streams byte-identical**, 0changed/0base-only/0ambiguity. Nine new streams
  all matched complete runtime Native output. ZIP framing/timestamps are ignored
  only for input identity, never for G-code comparison.
- Required-Native workspace/all-features at exact018:
  run `afe95e19-3f02-4c9f-aea2-d93752d51b55`, 8135PASS/0FAIL/7 existing skips,
  exit0. Initial workspace launcher exit126 was a non-executable script, before
  Native execution; preserved separately. The unchanged script was invoked with
  bash, without altering source, inputs, environment flags or assertions.

Coordinator authorized production-zero carry-forward through `eddfff507`
(Arachne prism tests/docs and Native-product deletion). After the running
workspace finished, rebase succeeded with no production-src delta to the above
candidate. Rebased implementation HEAD:
`e15f6466594ca42be50fe5659f73958cf8469db0`.
The historical seeded/workspace runs above did **not** execute on that newer
HEAD. No new official-Native epsilon run is claimed.

Post-edd rebase clippy (workspace/all-target/all-feature, warnings denied),
core+ares-wasm all-feature wasm32 check, rustfmt and directed CLI binary
compilation all exited0 at e15. Binary compilation is not Native execution.
Coordinator subsequently authorized production-zero carry-forward through
0588388e7 (travel fixture test migration/docs and Native-product deletion).
Final production-zero rebase succeeded to implementation/documentation HEAD
`38a040d1276b538c0f04612a88432d80c6386a88`; production-src diff from e15 is empty.
Workspace/all-target/all-feature clippy with warnings denied, rustfmt and
four-test directed CLI binary compilation all exited0 on that HEAD. Compilation
is not Native execution. The core/WASM check at e15 is source-equal carry-forward,
not a rerun on38a. The following final document-only revision changes no Rust,
tests or fixtures; all historical run bindings above remain unchanged.

Ready for sole-coordinator REVIEW; no approval, push or merge is claimed.
All streams, process diagnostics, exact receipts and byte audits stay in /tmp.
