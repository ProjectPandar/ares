# ARD: range-dependent PrintRegion rewrite

Scope: Stage2 first slice: explicit-height wall_loops region ranges.

1. Stage2 means **region-override ranges**, not the painted-profile
   Stage2 named in the earlier height-profile proposal. Painted profile remains
   separately deferred, along with SlicingAdaptive and activeZAA.
2. Owning upstream module is PrintApply.cpp::LayerRanges and
   generate_print_object_regions/get_create_region, together with
   PrintObject.cpp::region_config_from_model_volume and
   PrintObjectSlice.cpp::slices_to_regions. Port their existing data/control
   semantics into core, not an independently designed range pipeline.
3. Preserve one object-wide table of config-distinct PrintRegions, deterministic
   first-created IDs, and ordered range-specific volume/parent/region mappings.
   Range routing selects the mapping, not a new region table or cloned options
   per layer. Height-only stage1 equivalence remains unchanged.
4. Existing project::region_graph is the partially ported owning module for
   config dedup/parent modifiers. Expand its use rather than create another
   registry. Current project_slice/volume_regions single-candidate graph and
   region_slices single record list are temporary partial shells; replace them
   with source-owned range mappings. Remove their obsolete assumptions rather
   than add fallback paths around them.
5. For portable Native slice witnesses, every actual range carries explicit
   layer_height, including ranges changing only region options. Missing height
   hits upstream Slicing.cpp missing-option dereference UB. Do not fabricate a
   default, emulate a crash, or expand pure-None/mixed acceptance without a
   separately source-proved selection path. This stage removes region-override
   rejection initially only for explicit-height wall_loops overrides. Other
   region options retain their capability boundary pending independent TDD.
6. Core remains bytes-only/platform-neutral on browserWASM/Windows/macOS/Linux.
   No I/O/UI/OpenGL/terminal, no independent slicing in CLI/WASM adapters, no
   candidate crate or extensible strategy. Reuse existing mesh slicing and flows.
7. Independent Native runtime RED → test-first GREEN. No production or runtime
   probe in this spec-only change; no ignored test or broadened byte normalization.

8. G-code template total_layer_count uses the export's physical print-Z count:
   GCode.cpp:2487-2529 derives m_layer_count from object/support layer Z values,
   and :2890 exposes it to the shared placeholder parser. The existing Rust
   export schedule owns the merged count; pass it through all template contexts
   rather than count region-major perimeter records. Machine-start must not
   override it with a first-object record count. This correction does not change
   the comparison contract, M73, or actual layer geometry.
9. PrintObjectSlice.cpp:244-266 epsilon-aware range selection is a separately
   required source slice with its own RED/GREEN and commit, not included in this
   first Stage2 implementation. Do not claim epsilon-boundary parity from the
   constant-height wall-loop witness. Spiral mode with region overrides retains
   the unsupported boundary (upstream multi-range slicing asserts !spiral_mode).

10. Empty physical layer lists must fail at the slice-completion boundary, before
    perimeter traversal and export scheduling (`PrintObjectSlice.cpp:866-867`).
    Rust performs the existing no-layers rejection immediately after top-empty
    trimming. Region-major record traversal must not compensate for an invalid
    zero-layer object with a step-size special case. The retained zero-height
    regression test requires the original `no layers were detected` error.

Admitted2.4.2 identity: checked source equals seeded source store byte-for-byte.
PrintApply.cpp SHA256 a067414959c475d23f782ebac9f5a185b63012304aced66974726d95a95daf20;
PrintObjectSlice.cpp SHA256 d1aece12e7189353aa9a5d8ae6b1f0f320087c0d2574d7da64fe372eca076021.

See [Stage2 spec](../superpowers/specs/2026-10-08-range-dependent-print-regions.md).
