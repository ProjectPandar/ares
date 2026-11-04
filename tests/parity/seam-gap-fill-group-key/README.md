# Seam-gap fill-group key input witnesses

SPEC `0b5aa964a`, owning Orca8500fcd `Fill.cpp::SurfaceFillParams::operator<`
and owner-region clipping assignment at1279. Production computation/consumers
remain unchanged during RED.

These four input-only ZIPs are the existing independently probed projects,
byte-for-byte, without rebuilding geometry/settings. Each has ten members;
all eight non-settings members are unchanged from the original passing
compatible-region fixture. The fifth baseline is **reused in place** from
`../fill-group-flow-ratio-key/equal-density.3mf`; no duplicate is stored here.

| Input | Variation | SHA-256 |
|---|---|---|
| reused equal-density | global10%, no local override | `89a9b7babcf153df4377c809064d6036668c3e4667bc727302771d6630fe339b` |
| seam-equal | global10%, local10% | `03cfd43caac4c9c5661076a414885ac6f8bceb6cb8414483f9845acf56ab078f` |
| seam20 | global10%, local20%, target | `93d3b3ffae0821609f1052388e4f0313e8a7e2fae3ac43657539984537932c08` |
| global-seam20 | global20%, no local override | `0dd52f9e193f76ea9aa531c2aeeff72727ea3f6bfa3842eb9188a3b9f667b14d` |
| absolute-equal | global10%, local0.04 mm | `7620eafdbeb793c46cfcf7745e3e8c94d5bdbd76cd8f7e8aa07f5e2ccd6dde8b` |

The absolute control retains distinct region configs but equivalent scaled
clipping. Native first-owner projection uses the retained Flow nozzle0.4f,
not extrusion width/spacing. Tests do not inspect private grouping state:
they reuse shared fresh Native/API/CLI repeats and strict complete-output
assertions, with public input/footer/positive Bottom activity admission.

All five tests assert PASS; before the one-key GREEN,
only seam20 fails in both official and seeded modes. No inverted assertion,
Native G-code golden, ignored output, fallback or unsupported guard.
