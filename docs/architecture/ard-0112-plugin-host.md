# ARD-0112: Slicing pipeline plugin host

Status: accepted (2026-11-28). Tracking: [plugin-system.md](../plugin-system.md).

## Source boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) already splits
its plugin system into a `libslic3r` seam and a GUI-side host. Ares ports the
seam and replaces the host.

| Upstream | Role | Ares destination |
| --- | --- | --- |
| `PrintConfig.cpp:6007–6011` `plugins` (`name;uuid;capability` manifest) | project/preset key | already defined in `options/registry`; still inert |
| `PrintConfig.cpp:6013–6020` `slicing_pipeline_plugin` (`plugin_type = "slicing-pipeline"`) | selects capabilities | already defined in `options/process_options/print_source.rs`; still inert |
| `Print.hpp:132–138` `SlicingPipelineStepPlugin` | 12 in-pipeline steps + `psGCodePostProcess` | `ares-core` step enum |
| `Print.hpp:1061–1064,1491–1496` `SlicingPipelineHookFn`, `set_slicing_pipeline_hook_fn`, `run_pipeline_hook` | injected dispatcher; `libslic3r` has no plugin dependency | `ares-core` host trait supplied by the caller |
| `Print.cpp:3272–3275` | active iff a dispatcher is installed and `slicing_pipeline_plugin` is non-empty | same rule |
| `Print.cpp:3419–3423` `hook_after` and call sites `:3543,:3629,:3909,:3926` | fire only when the step was recomputed in this pass | same rule, keyed into step invalidation |
| `GUI/PostProcessor.cpp:266–290` | `psGCodePostProcess` edits the exported file | byte-in/byte-out step after export |
| `plugin/PluginHooks.cpp:72–140`, `plugin/PluginManager.cpp:137` | Python dispatcher, installed only by the GUI plugin manager | replaced by the WASM host below |
| `plugin/host/PluginHostSlicing.cpp:115–363` | data exposed to Python: surfaces writable, extrusions read-only | replaced by the interface below |

`src/OrcaSlicer.cpp` (CLI) never installs a dispatcher, so Native CLI output
ignores both keys. Ares keeps that default: a project that names plugins
slices exactly as today unless the caller explicitly attaches a plugin host.

## Decisions

1. **One plugin interface, in core.** `ares-core` exposes one public Rust
   plugin interface: the step enum, the data each hook receives and the edits it
   may return. It is the only contract Ares maintains; every other form is an
   adapter onto it.
2. **Native Rust plugins.** A Rust plugin can implement the core interface
   directly and be compiled into the application. It runs at native speed on
   every platform, including the browser build, but has no hot reload.
3. **Plugin host crate.** A dedicated crate (`ares-plugin-host`) loads WebAssembly
   and JavaScript plugins, adapts them to the core interface, and owns hot
   reload. The WASM call encoding and the JS object model are projections of
   the core types maintained only there. Use the same JS engine on every
   platform and define portable numeric behavior; pinning the engine alone
   does not guarantee identical output bytes. `ares-core` stays platform-neutral,
   never reads plugin files and has no WASM or JS dependency; the CLI and
   browser adapters attach the host.
4. **Sandboxed, deterministic plugins.** Every hosted plugin runs in a sandbox
   owned by `ares-plugin-host`. It gets no network, clock, randomness or
   threads, and no access to the real filesystem. Its only file access is a
   per-plugin virtual root (decision 10). Each hook is a pure function of the
   data the host passes for one layer or object plus the files it read; hooks
   run in the order of `slicing_pipeline_plugin`. NaN payloads returned by a
   plugin are canonicalized. Output must not depend on core count.
5. **Hot reload.** For hosted plugins, identity is the hash of the module
   bytes or JS sources. The identity is part of the configuration compared by the
   PrintApply invalidation, so replacing a plugin recomputes from its first
   hooked step and keeps earlier steps. File watching and reloading belong to
   `ares-plugin-host`.
6. **No plugin, no change.** With no host attached or an empty
   `slicing_pipeline_plugin`, output is byte-identical to the current code and
   every parity suite is unchanged. Output produced with plugins carries no
   OrcaSlicer parity claim.
7. **Perimeter extension.** Upstream exposes extrusions read-only, so its
   plugins can only alter perimeters by rewriting G-code text, where the
   perimeter index is lost. Ares adds one non-upstream extension at the
   `posPerimeters` seam: read-only inset index per perimeter path, and
   writable per-path Z, height, flow multiplier and perimeter order, plus
   duplication of a path, honoured by G-code emission. Its exact data model is specified with its first
   consumer.

8. **Plugin options.** A plugin declares typed options (key, type, default,
   range or enum values, label, tooltip, and the preset scope print, printer
   or filament). The host validates values against that declaration and hooks
   receive the resolved values; the UI renders the declaration generically.
   Values are stored in the upstream keys `print_plugin_config_overrides`,
   `printer_plugin_config_overrides` and `filament_plugin_config_overrides`
   (`PrintConfig.cpp:1300–1313`, already inert keys in Ares) as entries of
   capability id, plugin version and JSON values (`plugin/PluginConfig.hpp`
   `CapabilityConfigEntry`). Selections use upstream `slicing_pipeline_plugin`
   and the `plugins` manifest (`name;uuid;capability`). A saved 3MF therefore
   carries plugin selection and options in its project config, and OrcaSlicer
   keeps those keys. Option values are part of the configuration identity,
   so editing one recomputes from the plugin's first hooked step.
9. **Hosted plugins travel inside the 3MF.** Saving a project that uses WASM
   or JS plugins embeds their code, so the project slices the same anywhere.
   The project also records each plugin's capability reference, version and
   content hash; a mismatch is reported and never replaced by another build.
   Plugins run only inside slicing steps, never when a project is opened, and
   always in the sandbox of decisions 4 and 10, so an embedded plugin runs
   without a separate trust prompt. Execution is bounded by a deterministic
   fuel budget and a memory cap. To fail at the same point on every platform,
   use the same metering semantics; fuel counts from different runtimes are
   not interchangeable. JS runs on a memory-safe engine or inside the WASM sandbox,
   never on an unsandboxed C engine. A plugin declares the hooks it uses;
   undeclared hooks are never called. The G-code a plugin produces is not
   policed; it is treated like custom G-code in a project. Native Rust plugins
   cannot be embedded; a project that needs one records only its reference
   and version.

10. **Plugin file sandbox.** Each plugin sees one virtual root and nothing
    outside it; `ares-plugin-host` resolves every path inside it and rejects
    absolute paths, `..` and links. The root has two areas:
    - `/package`: read-only files shipped with the plugin. They are part of
      its content hash and travel with it in a 3MF.
    - `/data`: private read-write storage for that plugin only, with size and
      file-count quotas. It is never shared between plugins, never embedded in
      a 3MF and can be cleared by the user.
    During a slice, `/data` reads see a snapshot taken when the slice starts
    and writes become visible to the next slice, so parallel hooks cannot
    observe each other. The hash of the `/data` files a plugin read joins its
    identity for invalidation. The adapter supplies the backing store (an
    application data directory natively, browser storage in the web build);
    `ares-core` never sees it.

11. **Runtime and metering.** Native builds run hosted WASM plugins on
    Wasmtime; the browser build runs them on the browser's own WebAssembly
    engine. Neither engine's built-in fuel is used. Before instantiation,
    `ares-plugin-host` rewrites each module: it inserts one fuel debit per
    straight-line code segment and sets the linear memory maximum. Both engines
    then run the same instrumented bytes, so a plugin exhausts its fuel or
    memory at the same point on every platform. Fuel bounds the guest
    instruction count, not wall time. Bulk memory, table operations and host
    calls are charged statically. The linear memory cap does not cover tables,
    stacks or host allocations, so those need their own limits. Measurements:
    [runtime spike](../plugin-runtime-spike.md#metering-follow-up).

## Rewrite rule amendment

`AGENTS.md` requires every milestone to be an upstream rewrite slice. Plugin
features (for example staggered perimeters) have no upstream implementation.
`AGENTS.md` now carries this exception:

> Plugin modules are exempt from the rewrite rule. Core changes for plugins
> are limited to porting the upstream plugin seam plus the extensions recorded
> in ARD-0112; with no plugin attached, output stays byte-identical, and
> plugin output makes no OrcaSlicer parity claim.

## Verification

- Every existing parity and non-parity suite passes unchanged with no plugin.
- End-to-end: a sample plugin at each step class (post-process, perimeter)
  sliced through the CLI and the browser build produces byte-identical
  G-code, retained as a repeatable artifact.
- Core-count invariance (1/2/7/16) with plugins attached.
- Replacing a plugin during a session recomputes only from its hooked step.
