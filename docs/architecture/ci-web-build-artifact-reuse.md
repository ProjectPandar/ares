# Reuse WebAssembly artifacts in the web build

## Boundary

This changes how `scripts/ci/run-web-build.sh` prepares the browser test bundle,
not how Ares slices. The metadata audit still checks the Ares registry's
`ares_core::print_config_def_json()` export against `PrintConfigDef` and its
`init_fff_params()` in OrcaSlicer v2.5.0 `src/libslic3r/PrintConfig.cpp`. Runtime
behavior, the upstream sources, and changes to the registry itself are out of
scope. The existing Rust example and metadata verifier remain the audit entry points.

## Decisions

- Reuse normal WebAssembly bindings only with the exact Pages package key. A miss
  rebuilds with the same release profile as Pages. Use a separate exact key for
  the panic-probe bindings and their bundled JavaScript, including the Rust build
  inputs as well as the TypeScript, Vite, and toolchain inputs. Never substitute the
  probe for the shipped module. Build the normal module first when both miss so
  the probe can reuse its Cargo dependencies.
- Keep the release host CLI and metadata example, but disable cross-crate LTO and
  let Cargo split code generation for these two binaries. Do not change the
  profile settings used by the shipped module. Run the three tests that generate
  browser comparison inputs even when workspace tests also run; they retain all
  assertions and generate the bundle's JSON from the current checkout. Use the
  unoptimized host profile without debug info for this artifact-producing pass.
- Keep the exported metadata comparison, actual-binding TypeScript check, web
  build, and desktop shard completeness check. Move no browser tests. Remove only
  format, lint, web unit tests, and generated-metadata sync from the web build;
  `fast-checks` runs the same commands for both pull requests and merge groups.
  When the merge group reuses pull-request checks, it first requires a successful
  `fast-checks` result for the identical tree. A group whose scope job fails or
  whose required checks are cancelled cannot qualify.

## Verification

Run the build both without cached outputs and with both exact outputs restored.
Unpack the resulting archives and check for the same CLI, metadata example,
comparison inputs, probe, normal bindings, and package and app distributions.
Compare the CLI's G-code and the emitted metadata and fixture JSON with those
from the original build. Record cold and warm times separately: the cold path
still pays for the unchanged production WebAssembly build.
