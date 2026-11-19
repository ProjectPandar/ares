# Release oracle for the ordered Voronoi incident diagram

The incident test observes the default repair dispatch from OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), not the previous beta.
`src/libslic3r/MultiMaterialSegmentation.cpp:1665–1666` calls the colored-line
instantiation of `Geometry/Voronoi.cpp:34–73`. The observer also supports the
line instantiation used by `Geometry/MedialAxis.cpp:463,471`. The detector and rotation
boundaries are `Geometry/Voronoi.cpp:172–187,296–366`.

The release seed-replay recipe provides the matching ELF, separate DWARF,
resources and build-source tree; use the clean release checkout for the source
identity and source-byte checks. The fixed entry addresses are x86-64 virtual
addresses from the recipe's debug symbols for the two `construct_voronoi`
instantiations, the corresponding `detect_known_issues` instantiations, the
rotated-segment detector and the two rotation instantiations. The addresses are
valid only for the identified ELF/DWARF pair, not for the official executable
or a new build with different offsets. The test remains a seeded observation
even when run alongside official consumer parity tests.

Only the oracle identity changes. Keep the two fresh captures, input and source
checks, raw layout validation and exact ordered diagram comparison. A revision
string alone would leave the old breakpoint addresses and generator identity in
use; both must match the release producer before the backend is compared.
