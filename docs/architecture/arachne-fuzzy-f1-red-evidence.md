# F1 public Native RED

SPEC308583e3a approves single-group whole coverage as F1, separately from F2
effect masks and ordered LineSplit. Production remains unchanged from c22f5a81d;
no old unit test, helper, assertion or guard is changed by this RED commit.

Fresh run: `/tmp/worker3-fuzzy-f1-public-red-ypcs1mmh`.
`plan.json` records complete per-leg environment/command, input manifest hash,
base, CPU12 affinity and jobs2/threads2. Legs ran sequentially with separate
fresh Native caches/artifact roots, after the W4 exclusive window ended.
`red-evidence.json` audits every admission/error, comparison and raw Native SHA.
All Native/API/CLI outputs remain outside Git; repository 3MFs are inputs only.

| Mode | Tests | Native admissions | Strict matrix contracts | Strict PASS | Diagnostic matrix | RED |
| --- | --- | --- | --- | --- | --- | --- |
| official | 30 | 30/30 | 13 | 4 | 16 | 25 matrix + Context |
| seeded | 30 | 31/31 | 29 | 4 | 0 | 25 matrix + Context |

Both nextest legs exit100 with4 passed/26 failed. Every matrix RED admits
Native, then the validated byte API returns
`UnsupportedProjectFeature("fuzzy_skin")` and the real CLI exits nonzero.
Context's five public API attempts (seed0, seed1, repeated0, independent0/1)
reach the same existing rejection. Seeded Context has two successful Native
seed references; official has one stochastic diagnostic reference.
The four passing controls are Disabled, None, zero thickness and sub-threshold
thickness. Their API/CLI outputs have identical raw SHA and both strictly match
Native ordered generator bytes in each mode.

Official stochastic cases never invoke the strict comparator and never count
as Native parity PASS. Required comparison and actually performed comparison
are distinct artifact fields; unsupported public calls perform no byte parity.
Seeded and deterministic Ripple/inactive contracts use the unchanged existing
ordered generator comparator. No normalization or chance retry is introduced.

Inputs cover all six noises and three modes, contour/hole/inset eligibility,
first-layer control, Ripple phase/real slice_z, open-neck varying width, short
padding and task Context. Native effective fuzzy settings are independently
checked against authored inputs before public Ares calls. PrintObject.cpp:
3707–3708 owns zero/sub-threshold regional normalization; these are inactive
controls even when the global footer retains External.

Source owners: PerimeterGenerator.cpp:378–387; FuzzySkin.cpp:223–291,
345–412,415–492,685–694. Closure/minimum-width preflight and the explicit
PerimeterGenerator.cpp:534–540/NDEBUG ruling are recorded in
[preflight](arachne-fuzzy-f1-native-preflight.md) and the SPEC addendum.
The two Native Flow::spacing admission failures are excluded from parity.

The interrupted run `/tmp/worker3-fuzzy-f1-public-red-74s0mkl9` is VOID.
Its pre-pause official26 failures are reference only, never RED evidence;
the stopped process tree was killed without resumption. This fresh run is the
sole public RED evidence here. New test modules are80–127LOC; fixtures and
documents also remain under400LOC. Test-target strict clippy and fmt exit0.
F1 production/GREEN still follows completion of the alternate_extra_wall merge.
