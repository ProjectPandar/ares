# TiQ2 first-layer boundary/cache history

Read-only extension `tiq2_travel_history.nix` of the qualified travel family.
It logs first-layer `init_layer`, internal construction/reconstruction input,
and cached output. No route or cache predicate is changed.

## Qualified result

The build and the exact-fixture seeded slice both exit 0. Package:
`/nix/store/dn62si3vdhklizgyd0zljdvpljrnngza-orca-slicer-seed-replay-2.4.2`.
Complete bytes equal
seed oracle `1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv` after ONLY replacing the
generated-by banner. Both816581 normalized bytes, SHA256
`8e417e46deaef9e48f9390cf9ae0412b7977722b7722fdfcf56c2eb7cd467001`.
Raw oracle SHA is unchanged3d5b2f3f7a66b064a20e3a55024320f7911165625210fd15bd7a7ab7584a9ca7.

The ordered ledger is:

1. INIT_LAYER object/support=0, spacing437079, regions1, old internal empty.
2. CONSTRUCT support=1, spacing400000, regions0, raw start
   (-9595486,-8816201), end(-8642429,7955622). Support lslices empty;
   below-object lslices are the square±5000000.
3. CACHED internal square±4400000.
4. Target object travel support=0, current spacing437079, SAME internal cache;
   interior point(4399900,4301461), origin165,165 ->169.3999,169.301461.
5. PRE_RETRACT and FINAL retain that path and last_pos. No intermediate
   init_layer or cache reconstruction occurs.

## Owning source and Rust mismatch

`GCode.cpp:5343-5346` initializes once per instance before support.
`:5387` temporarily switches `m_layer` to SupportLayer; `:5429` restores
it without another `init_layer`. `AvoidCrossingPerimeters.cpp:492-507`
falls back when the active layer has no region flows. `:480-488`
`get_default_perimeter_spacing` averages scaled nozzle diameters over
`PrintObject::object_extruders()` (`PrintObject.cpp:3784-3803`), not all
configured slots or only the selected support slot. `:1100-1134` builds
support boundary from below-layer lslices and no SupportLayer top regions.
`:1252-1264` retains the internal cache when its expanded bbox contains the
subsequent endpoints.

Rust `layers/support_entry.rs:38-56` copies `EntryGeometry::view` of the
object. `layers/entry.rs:65-73` supplies the object's region-flow spacing;
it is not replaced with SupportLayer's default nozzle average. This yields
the deeper boundary responsible for X169.344. The object step follows
support in the same `tool_entry` (:164-192), retaining that wrongly built
cache. Do not delete all instance cache resets: the reset at tool entry
corresponds to the real upstream init_layer.

## Fix/RED boundary before production changes

Keep the full TiQ2 complete-byte RED unmodified. Test the support→object
sequence using the captured initial safe zone, support default spacing,
shared internal cache, exact raw endpoints and expected interior point.
Failure modes: inheriting object spacing; averaging inactive configured
slots; using selected support nozzle instead of object-printing set;
reinitializing on support→object restoration; inheriting model top surfaces
into SupportLayer's empty-region boundary. Do not hardcode0.4, tweak a
waypoint, add a fixture guard or change Clipper/router rounding.

No skipped or initialization-failed test is counted as coverage.
