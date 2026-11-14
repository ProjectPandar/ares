# Initial plate for projects without model settings

## Source and destination

At OrcaSlicer `40ecc66f7`, `src/OrcaSlicer.cpp:4699-4701` loads plate
metadata only when the importer supplied plates. Otherwise the initial
`PartPlateList` plate remains: `src/slic3r/GUI/PartPlate.cpp:5979-5985`
constructs it at the origin. `OrcaSlicer.cpp:5617` reloads object membership;
`PartPlate.cpp:7490-7515` iterates model objects and instances in order,
assigning each to the first intersecting plate using its convex-hull box.
`PartPlate::intersect_instance` at `:2665-2688` owns the intersection rule.

The Rust destination is the existing rendering-neutral `ares-core::plater`
plate-list port, invoked by project slicing before final plate selection.
The archive importer continues to expose the metadata actually present;
it does not invent an imported plate or loaded instance identity.

## Included behavior

When no plate metadata was imported, initialize plate 1 at the origin,
reuse the existing `PlateList::reload_all_objects` geometric assignment,
and expose that membership to the existing selected-plate slicing path.
Instances keep their allocated identities and transformations. Existing
projects with plate metadata keep their current behavior. The existing
print-volume and print-validation steps still determine printability.

## Deferred behavior

No arrangement, centering, extra plate creation, import metadata synthesis,
viewer runtime, or missing-project-settings defaults are added. GUI loading
and export are not changed. Existing Ares plate-selection scaffolding is
only a compatibility shell around the cited `PartPlateList` concepts.

## Failure modes and verification

A synthetic cube archive without model settings must slice plate 1 rather
than fail lookup. A public validated slicing test compares its complete
G-code with Native CLI output using the existing generator-only comparator.
Existing optional-metadata import tests must still expose empty imported
plate metadata, and existing plate-session tests must remain unchanged.
