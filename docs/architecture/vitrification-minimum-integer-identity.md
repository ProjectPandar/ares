# Vitrification minimum placeholder integer identity

## Source-owned follow-up

This is a bounded libslic3r rewrite correction to the placeholder-double chain,
not a new Ares template policy. Upstream pin:
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

`PrintConfig.cpp:2917-2923` registers `temperature_vitrification` as `coInts`;
`PrintConfig.hpp:1340` stores `ConfigOptionInts`. `GCode.cpp:3017-3019` computes
an `int` minimum over writer extruders' values. `GCode.cpp:3040` registers
`min_vitrification_temperature` as `ConfigOptionInt`, not ConfigOptionFloat.
`Config.hpp:968-972` serializes that integer without double precision rounding.
Legacy `[...]` expansion calls scalar `serialize()` at
`PlaceholderParser.cpp:932-936`; expression expansion preserves coInt through
`set_i` at1041 and TYPE_INT `std::to_string` at310. TYPE_DOUBLE ostream conversion
at311-321 remains a separate operation even for integral-valued doubles.

Rust destination is ONLY the two existing derived-placeholder writebacks:
- `project_slice/gcode_emit/placeholders.rs` runtime minimum binding;
- `project_slice/gcode_emit/layer_gcode.rs` layer template binding.

Both must construct `Value::Integer` for this integer placeholder. The existing
numeric value is already an integer; do not invent an extra validated-i32 check,
guard, saturation, fallback, or unsigned conversion. No generic Config retyping.
Preserve each site's existing value selection and evaluation. Adjacent existing
writer-extruder/minimum-vs-first-element selection behavior is deferred; this
slice ports the scalar integer identity, not a claim of full multi-tool lifecycle
parity. This extends the earlier double-render-only constructor freeze at these
two named bindings only. All other constructors remain unchanged.

## Existing RED and minimal correction

The existing, unchanged
`option_coverage::vitrification::vitrification_signed_values_reach_start_and_complete_output`
is the regression seam. On the failed integration26a87adfe and the double chain
its negative case compares full ordered generator output and reports Native
`;VITRIFICATION -123456789` vs Ares `;VITRIFICATION -1.23457e+08`. The domain test
and110 control are retained, as is the existing nonnullable INT_MAX rejection.
No new tests, changed expectations, fixture regeneration, masks or tolerances.

Seven-point diagnosis proved base/volume/zero PASS, double/name/ironing/negative
FAIL; first production trigger is a89643f90, replay of360e9aa90. The correct
all-Number precision-six formatter exposed a preexisting Number binding.
Relevant parser/binding/test files are identical between the original double
base47821c760 and integration basead5111f0b: this is incomplete closure against
an old Ares compatibility binding, not new-main brim interaction.

The correction changes only the two integer writebacks. Number formatter,
Integer/String arms, generic parser, helpers, tests, input and expectations stay
unchanged. Passing this narrow module does not establish whole-template parity.

Core remains platform-neutral Rust; no filesystem, process, terminal or native
formatting dependency is added. No browser runtime or multicore claim.
