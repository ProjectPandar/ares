# OuterOnly brim inputs

Approved SPEC a68680255 ports Brim.cpp:448-570,661-711,903-904.
These eleven 3MFs contain authored input meshes and settings only.
Run `python3 tests/parity/outer-only-brim/generate.py` from repository root.
The orthogonal mesh generator preserves deterministic ZIP metadata, verifies
oppositely paired triangle edges and positive signed volume, and saves hashes,
contours, holes, counts and effective requested settings in provenance.json.

One object/instance/model-part volume/group/tool, two Classic walls, zero
infill/top/bottom and disabled support/tower/skirt/EFC. Two non-nested islands
are disconnected components of one volume, each with a hole. Width0 and
sub-pair width are inactive controls; no-hole is the unchanged active control.

The clipped-contact input uses a printable polygon with an open notch at
global y125..126.2 ending at x130, beyond the model's outer brim. The model
ends at y125. Its upper printable brim fragment is separated by the notch
from the footprint and other brim fragments. Native admission and the actual
retained/excluded motion witnesses determine whether this input counts as
contact coverage; no result is claimed by generating it.

Tests save full Native, validated API and real CLI streams, Native effective
footer/launch, hashes, admission/comparison receipts and Brim XY/E witnesses
under ARES_PARITY_ARTIFACT_ROOT. OuterOnly requires exterior brim and no
hole-side or detached-contact-region extrusion. Witnesses supplement the
unchanged generator-only ordered byte comparison, never replace it.

Use canonical control-set-v2 jobs2/threads2, load<30 and a fresh cache/root
per mode/run. Existing InnerOnly/OuterAndInner/N/O/E and bed controls remain
unchanged. Native G-code is never fixture data.
