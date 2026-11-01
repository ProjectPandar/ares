# Beta tree wall-count legacy rewrite boundary

Upstream revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
Destination base: `abd1f70b975c7198553392fe22538d1ac182a34b`.

PrintConfig.cpp:7958-7965 defines tree_support_wall_count as coInt,
minimum0, maximum2, default0;0 means auto. This definition remains intact.
PrintConfig.cpp:9839-9840 handle_legacy rewrites the string token -1 to0.
The rewrite is not permission to pass a negative wall count to runtime.

Port only that exact conversion to the existing Rust token and typed legacy
adapters in options/legacy/token.rs and options/typed_legacy/actions/rules.rs.
The current adapters are compatibility shells for upstream handle_legacy,
not alternate option semantics. Do not change numeric range validation,
metadata, support path generation, loop-count heuristics or comparator rules.
After normalization, existing zero/auto semantics apply (SupportParameters.hpp
141,203-205; TreeSupport.cpp1538,1816; TreeSupportCommon.hpp101).

## Public Native-first test

Use the unchanged historical Ender3 archive with only project_settings.config
rewritten at runtime to author tree_support_wall_count="-1". Native
--export-settings must succeed and export0. Compare that authority to public
SliceOptions input carrying the same string legacy token and public
load_project importing the exact same authored archive, including typed
process-object serialization. The canonical single-group DTO is not a legacy
import boundary and must not gain independent handle_legacy behavior. Preserve modified input, input
hash, Native status/identity and public values externally; commit no Native
output. This observes legacy import normalization, not active support parity.

Failure boundaries to keep distinct: Native rejects/does not rewrite the
legacy input; either public adapter leaves -1; adapters disagree; or downstream
support/G-code still differs after normalization. A Native command failure is
not a qualified RED. No invented truthiness or numeric/string coercions.

Use the accepted49397 ledger's tree-wall-count failures as the before
inventory, map B12 names explicitly, rerun only surviving affected failures
in both producers after GREEN, and report downstream failures honestly.
No FULL, min relaxation, new ROOT registration or legacy engine fallback.
