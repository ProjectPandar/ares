# Loaded-system `inherits_group` public parity fixture

The fixed input project and `positive/*.json` retain the original source bytes;
`manifest.json` pins them and the derived `control/*.json` files.
`generate_control.py` blanks only the three `inherits` strings for the
standalone typed-file bypass case. The manifest's upstream revision records
that input's original provenance.

On OrcaSlicer v2.5.0, a typed file with `from=system` and a nonempty parent
must be supplied through its owning vendor manifest. The test stages the
positive leaves at their canonical paths under private copies of the pinned
preset trees. A second positive invocation independently resolves and slices
the same inputs. The equivalent-parent control stages the same leaves but
points to identically configured, differently named parent definitions in the
same manifests. See
[`loaded-system-manifest-fixture.md`](../../../docs/architecture/loaded-system-manifest-fixture.md)
for source ownership and the expected equality checks.

The original `control/*.json` files now serve as a separate blank-parent
bypass control, compared end to end; Native does not complete these typed
files through the manifest. The three manifest-backed Native outputs must be
identical except for the generator identity. Both public Ares entries must
agree on raw output for every leg, and the complete Native/Ares comparison
keeps configuration, motion, arcs, time estimates and statistics. The test
also checks the Native system-source group (`;;`) and repeat stability.

Run in official and seeded modes with canonical reference flags and the
runtime preset supply. Each leg preserves its staged source files, Native
result and command capture, all complete streams and configuration maps.
No Native output is stored as fixture input.
