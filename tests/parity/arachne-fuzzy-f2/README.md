# F2 multi-group fuzzy inputs

Input-only authored modifier 3MFs, based on the F1 watertight meshes.
Generate from repository root with `python3 tests/parity/arachne-fuzzy-f2/generate.py`.
Every modifier explicitly preserves wall count, so perimeter compatibility
continues to group these parts while their complete fuzzy configs differ.

Cases cover None/External in both directions, two different active effects,
same-effect type merging, inner eligibility, later overlapping modifier priority
and open-neck variable-width source junctions cut through a non-vertex boundary.
All use deterministic Ripple: official and seeded both require complete ordered
generator bytes from the validated API and real CLI. No normalization/retry.
Model/part transforms and authored overrides are recorded in provenance.json;
Native raw output, commands, identities and API/CLI evidence stay outside Git.

Source: FuzzySkin.cpp415–473/696–814, Algorithm/LineSplit.cpp64–314.
