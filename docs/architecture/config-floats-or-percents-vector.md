# FloatsOrPercents: non-nil vector extraction and percent preservation

Status: source-audited proposal only. No runtime or production/test change.

## Owning upstream PrintConfig / Config boundary

Pinned OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `PrintConfig.cpp:2735-2747` registers `small_support_perimeter_speed` as
  `coFloatsOrPercents`, `ratio_over=support_speed`, `min=1`,
  no `nullable=true` assignment (definition remains non-nullable), default
  `ConfigOptionFloatsOrPercentsNullable{(50,true)}`. The nullable default
  object's concrete class does not change the definition's nullable flag.
  This is the sole non-nullable coFloatsOrPercents registration in the cited
  upstream set and is the concrete witness for this slice.
- `Config.hpp:1400-1429::ConfigOptionFloatsOrPercentsTempl<NULLABLE>` owns the
  vector/value/percent/nullable vocabulary; `type()` is coFloatsOrPercents
  independently of the NULLABLE template argument.
- Its `deserialize:1457-1479` trims comma-delimited elements. The exact `nil`
  branch is separate: nullable stores the nil sentinel, nonnullable throws.
  In the **non-nil branch**, percent is `find_first_of("%") != npos`, then a
  fresh `std::istringstream >> double` writes value; `{value,percent}` is
  pushed **without testing failbit**. Ordinary extraction failure writes0
  under the delegated C++ num_get contract; range overflow is a different case.
- `serialize_single_value:1490-1503` emits the stored finite value with a `%`
  exactly when the flag is true. `vserialize:1443-1454` exports each token.
- In contrast, scalar `ConfigOptionFloatOrPercent::deserialize:1381-1393`
  returns false on failbit before assigning value/percent.
- `Config.cpp:982-1054` routes project setting tokens into deserialize;
  `1564-1584` saves real scalar/vector serialization in public settings JSON.

The rewrite goal is this upstream vector-versus-scalar extraction distinction,
not an Ares parsing policy, overhang algorithm or slicing pipeline feature.
The PrintConfig registration is the concrete public Native witness boundary;
the ratio/default/minimum are not being redesigned.

## Exact Rust destination and scaffold disposition

`crates/ares-core/src/options/decode.rs::decode_config_value`, the
`(FloatsOrPercents, WireValue::Sequence)` arm currently calls the public scalar
FloatOrPercent decoder recursively for every element. Scalar failure becomes
an error for the whole vector. That is not the cited non-nil vector branch.

Proposed production boundary: only this arm and, if needed, a small private
per-element conversion beside it. For a non-nil element, retain the existing
floating extraction helper's numeric result regardless of its success flag;
compute percent presence independently from the whole element, and use the
existing FloatOrPercent JSON renderer. Do not call scalar failure policy from
this vector path. Keep the scalar decoder and shared lexer untouched.

The recursive scalar wrapper is a temporary compatibility shell around the
upstream vector concept; this slice replaces that dispatch for non-nil
members. Existing wire splitting, ordering and JSON object vocabulary stay.
No new public API, option kind, framework, dependency, FFI, fallback or
key-specific behavior. Core remains WASM/Windows/macOS/Linux-neutral.

## Nullable boundary is explicit, not inferred from the kind name

The chosen PrintConfig key is the sole non-nullable upstream
coFloatsOrPercents registration. A **non-nil** malformed token therefore
witnesses the vector extraction branch without certifying nullable sentinel
behavior.

Current public Rust FloatsOrPercents JSON is a vector of `{value,percent}`
objects; it has no nullable counterpart kind. Key-based decode resolves kind
but does not carry nullable metadata into this arm; its encoder similarly
parses non-null FloatOrPercent objects. Therefore native nullable `nil` versus
public JSON null support is a separate schema/dispatch boundary, not something
to silently add while fixing ordinary vector extraction.

**Included:** preserve current rejection of exact `nil` in this generic
nonnullable-shaped public path, corresponding to upstream nonnullable nil
rejection; do not reinterpret nil as an ordinary failed numeric element0.
**Not claimed:** parity of key-based nullable nil decoding. That existing
missing representation stays explicitly deferred. No fallback to0 for nil,
key-name exception, new nullable metadata routing or silently changed schema.
If review requires full nullable-key parity in this slice, expand the source
ARD before any RED or production implementation.

## Included non-nil behavior

- Failed ordinary numeric extraction still supplies its source numeric result;
  percent presence is retained even on failure. Source predicts `bad%` as a
  stored zero-percent member, unlike a scalar error. This is not a runtime
  observation or an expected-string golden.
- Non-percent failed tokens preserve percent=false.
- Valid absolute/percent values and suffix-prefix behavior keep their current
  extraction semantics; percent detection is presence anywhere, not only a
  trailing-character test.
- Preserve order, element count, percent flags and the existing typed JSON
  shape. Exact nil handling remains outside the numeric-failure branch.
- Do not force all failed conversions to zero: overflow/underflow/hex/locale
  and other helper limitations are not repaired by a vector dispatch change.

The proposal starts on main53948dd6a620adf843860c3c6d3b0b72008273ad. That base
still contains the earlier exponent helper; its separately approved exponent
repair is not silently cherry-picked here. The proposed `bad%` witness has no
numeric prefix and does not depend on incomplete-exponent semantics. Do not add an
exponent-failure case before its source dependency is integrated and routed.

## Proposed public Native-first proof after approval

1. Before production code, add one E2E in the existing config-block round-trip
   subtree, reusing Native settings export/project mutation and producer
   identity adapters. No top-level parity ROOT change or private helper test.
2. Actual pinned Native first: set the real registered
   `small_support_perimeter_speed` project token to `bad%`, run
   `--export-settings`; retain actual exit,
   streams, complete settings and exact Native serialized member. Then public
   decode-by-key plus encode-by-key must be compared with that observed token.
   Persist both Results before asserting; do not hardcode a zero-percent
   expected string or copy the deserializer into the test.
3. If Native rejects or normalizes before this class is reached, stop/review
   the witness instead of inventing a key, private C++/stdlib kernel, Native
   rebuild or substitute diagnostic. No missing-Native or zero-selection pass.
4. Submit the real RED and scope audit before production. After code approval,
   change only the non-nil vector dispatch; qualify absolute/percent success,
   failed extraction with/without percent, and scalar failure unchanged.
   Preserve nil non-coercion only as a public generic-path non-regression;
   do not label that a Native nullable-key nil parity result. No bulk lexer
   matrix or browser run.
5. Run the existing config-block round-trip family with its original complete
   comparators, retain control status deltas, fmt/all-target/all-feature clippy
   and wasm32 checks. Capture repeatable source/input/producer/command/status/
   export/public-result artifacts; never commit generated Native goldens.
6. Keep files below400 lines and source/Rust destinations explicit; submit
   GREEN review before publication. No runtime is granted by this proposal.

## Deferred behavior and stop conditions

The twelve upstream nullable coFloatsOrPercents registrations that Ares
currently maps to scalar FloatOrPercent—`overhang_1_4_speed` through
`overhang_4_4_speed`, `internal_bridge_speed`,
`initial_layer_travel_acceleration`, `initial_layer_travel_jerk`,
`initial_layer_travel_speed`, `bridge_acceleration`,
`small_perimeter_speed`, `sparse_infill_acceleration`, and
`internal_solid_infill_acceleration`—are explicitly deferred. Their
scalar-to-nullable-vector registry migration overlaps the nullable-vector
registry work and is not part of
this slice. Nullable-key nil/null schema/metadata routing, new nullable enum kinds,
append/resize inheritance and all-value-nil semantics, shared exponent/float
lexer changes, scalar FOP policy, integer parsing, overflow/underflow,
signed zero/NaN/inf/hex/locale, JSON shape normalization, percent arithmetic,
ratio evaluation, PrintConfig metadata/default changes, loaded-range error
rendering/predicates, geometry/G-code/timing/UI/CI/cache work are deferred.
No new pipeline or generic parser consolidation is part of this rewrite.

Failure inventory before tests: wrong registration/type; nullable flag lost;
nil accidentally coerced to0; percent flag lost on numeric failure; scalar
failure policy unintentionally changed; Native token normalized earlier;
no actual Native execution; expected token fabricated; full control output
filtered; predecessor/helper dependency silently pulled in. Any exercised
prerequisite outside the boundary stops for review rather than broadening code.
