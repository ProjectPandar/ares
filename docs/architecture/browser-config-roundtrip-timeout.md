# Browser config round-trip: compile prerequisite before the test clock

Status: root cause and repair boundary approved; implementation/qualification pending.
Base: `5910df8eb0c2a9cab5958a1dcce7fdb34d075ecb`.
Upstream: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Existing source-owned witness

`GCode.cpp::GCode::append_full_config:8244-8353` writes whole
`ConfigOption::serialize()` strings through `cfg.opt_serialize(key)`
(`:8348-8350`). The fixed-precision wipe-tower coordinate line precedes the
ordinary serialization line, so the witness keeps the last key occurrence.
Rust behavior stays in `ares-core/src/options/encode.rs::encode_config_value_by_key`
and its existing decoder, exposed by
`ares-wasm/src/lib.rs::decode_config_value/encode_config_value:130-145`.
The JS worker transports strings; it must not become another codec.

The existing `runner/config_block_roundtrip.rs` real Native comparison supplies
whole strings. `ares-wasm/tests/encode_roundtrip.rs` supplies real stored-3MF
scalar/vector-element values. The browser spec combines both and compares
exact encoded bytes. This is a prerequisite repair for that existing rewrite
witness, not a new Ares pipeline feature or a demonstrated Rust codec defect.

## Reproduced cause

One unmodified desktop `web/e2e/configRoundTrip.spec.ts` execution on the base
failed its original300000ms timeout after about16.3minutes. Its trace shows
974157.125ms between before-hook completion and the first combined-value count
assertion, with **no config-page navigation or round-trip evaluation**. Both
result and Native temporary directories remained empty during that gap. The
`orca_parity` executable's build fingerprint finalized late in the interval.

The source runs synchronous `execFileSync("cargo", ["nextest", "run", ...])`
inside the timed test, before its first async page build. Native eventually
exited0 in3.642834seconds, and its733 whole values passed the Rust inverse
check. The stored-3MF1300 values also passed. After combining2033 values, the
first async yield observed the expired deadline; the browser codec never ran.

Thus the owning defect is **nested Cargo build/test preparation charged to the
browser deadline**. Node's synchronous child call blocks its event loop;
Playwright's timeout includes the test function/setup. The measured gap is not
claimed as pure compiler CPU time. Slow Native slicing, worker codec execution
and preview-server shutdown do not explain this reproduction: respectively
3.642834seconds, not reached, and not reached. Historical CI timeout messages
match, but unavailable historical traces do not prove their inner phases.

## Approved minimal repair

At the end of existing `web/package.json::prepare:e2e`, compile/discover the
**same target, profile, features, filter and build environment** as the spec's
later command, before Playwright begins:

```sh
cargo nextest list -p ares-cli --test orca_parity \
  -E 'test(=runner::config_block_roundtrip::config_block_encode_inverts_decode)'
```

`list` builds/discovers; it does not execute the Native comparison. Release CLI
and WASM test builds alone do not prepare this executable. Keep the in-test
`cargo nextest run` unchanged: absent/mismatched preparation must still build
or fail loudly, never skip. Native still runs in the real timed spec.

Non-negotiable: no timeout increase, `test.slow`, retry, comparator/fixture or
missing-Native policy change, saved Native JSON substitute, new preparation
flag/framework, Rust/JS codec change, CI configuration/cache/remote mutation.
Generic process supervision, other specs and new codec behavior remain deferred.

## Qualification

Run normal preparation with the barrier, then the unchanged spec **once** on
base+fix with normal affinity, original300second budget and no retries. Retain
trace and Native results. Bound the in-test Cargo phase by the trace interval
from setup completion to the first post-Cargo count assertion; retain build
fingerprints to distinguish compilation from Native runtime. Require actual
browser round-trip execution and a result with all actual input values and
zero differences. A discovery-only run is not GREEN. No shared-cache deletion
or manual warm-up retry is part of proof.

References: Node24 `child_process` synchronous process creation documentation;
Playwright1.63.0 `docs/src/test-timeouts-js.md`. No Rust production behavior is
replaced, and no independent slicer pipeline is introduced by this repair.
