# ARD-0059: Numeric range completion must include implicit endpoints

Upstream boundary: `libslic3r/Config.hpp:2475-2479`, `ConfigOptionDef`
initializes omitted numeric bounds to `-FLT_MAX` and `FLT_MAX`, not infinity.
`PrintConfig.cpp:2127-2135,2925-2931,4835-4841,6776-6783` supplies concrete
one-sided definitions. Integer storage and reserved nil values additionally
constrain serializable ranges (`Config.hpp:995-1085`); runtime constraints
must be source-cited and verified rather than inferred from small probes.

Rust destination is the native option coverage adapter's `domains`/`bounds`
projection and its `OptionOutcome` completion contract. This adapter is a
temporary evidence shell around upstream option definitions, not a new slicing
pipeline. No production slicing or G-code comparison changes are authorized by
this slice.

Non-negotiable: matching every generated local probe does not establish range
completion when an endpoint or full-range seeded sample is missing. Preserve
these comparisons but report INCOMPLETE with the missing evidence. Distinguish
an absent source assignment from an unsupported expression; neither can silently
become an unbounded complete range. `max_literal` is a GUI warning threshold,
not the legal absolute-value endpoint.
For explicitly bounded `coFloatOrPercent`, generate the absolute seed across
the same full min/max interval as the percent branch. Concrete source examples:
`PrintConfig.cpp:5844-5855` (`spiral_mode_max_xy_smoothing`) and
`6404-6413` (`support_threshold_overlap`). Width-schema extraction must use
the same whole-expression parser, not a numeric-prefix shortcut.

Preserve verified source-owned domains for skirt height, hot-plate temperature
and vitrification temperature. Keep Boolean/enum and explicit bounded coverage
contracts intact. Include behavior regressions and original-binary selected
sweeps whose comparisons succeed while their domain completion remains false.
Defer actually completing each previously omitted endpoint to follow-up
source/storage/runtime port slices. Rejection, crash or serialization loss does
not satisfy full-output parity. Correcting the gate does not complete the goal.
