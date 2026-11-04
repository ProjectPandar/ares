# ARD — continue the shared counterbore port through real Arachne handoffs

Status: proposed on main `5a9687ac020962f254b6262f2184f3345025c70f`.
[SPEC](arachne-counterbore-continuation-spec.md).
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Binding decisions

- `PerimeterGenerator.cpp::process_arachne:2123` shares
  `process_no_bridge:1728–1929` with classic. Reuse the existing Rust common
  geometry in `classic/counterbore.rs`; its module name is only a temporary
  storage/compatibility shell, not permission for classic walls on Arachne.
- Update pre-wall surfaces and preserve generated fill at the existing Arachne
  materializer. Counterbore fill precedes normal island fill, matching upstream
  `:1899–1929,2475–2533`. Retain separate no-overlap geometry and typed region
  output. Removing the guard alone is not support.
- The merged resolution prerequisite is authoritative: wall and infill
  simplification tolerances remain distinct. Do not duplicate its main-owned
  None test or repair ArcFitter, Fill or geometry to hide another parity failure.
- Production scope remains only `classic/preflight.rs`,
  `classic/counterbore.rs` and `arachne/materialize.rs` under
  `ares-core/src/project_slice/perimeters/`. More owner files need an amendment.
- Preserve original fuzzy mask domain (`Feature/FuzzySkin/FuzzySkin.cpp:415–473`),
  group discovery order, source metadata and RNG draws. No centering,
  interface-shell/classification, grid/Fill, Arachne backend/config or shared
  recovery edits within this slice.
- Keep byte-oriented platform-neutral API behavior on all four Tier1 platforms;
  Native execution/evidence stay in the CLI test adapter. No new stage/public
  API, core I/O, region[0], Unsupported, classic or Native fallback.
- Preserve source gate/no-op distinctions for None, lower slices and absent
  unsupported coverage; margin is the pinned upstream constant, not a new option.
- Public source-backed RED precedes production. Ten old rejection/lifecycle
  assertion changes need per-case exact diffs and a separate test commit.
  No assertion weakening, geometry replacement, ignored tests or comparator change.

## Consequences / deferred proof

The eight archived guard REDs are refreshed against merged main and stay on the
feature branch until GREEN. The merged main-owned None control is reused unchanged.
Nested-island-in-hole and compatible-multiregion public reachability remain explicit
coverage gaps; independent two-island geometry proves neither. Any unexpected
new-main mismatch stops implementation for source-owner analysis rather than
silently expanding the three-file allowance.
