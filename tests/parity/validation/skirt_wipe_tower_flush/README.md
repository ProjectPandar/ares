# Per-object skirt x wipe tower RED fixtures (W5)

- per-object-tower.3mf  sha256=d9ee50b057550861a5bcf7ef9fa62a90e08c54cb3c197ebf5255b6c4122e145b
  Derived from tests/parity/semm-ramming/R1.3mf
  (sha256=0b765abbf5dcc3d7196a5555dba9241fc6c5d55830e0b986cf9fbc9a62b00b9f,
  Creality Ender-3 SEMM, single nozzle, two filaments, prime tower
  enabled, objects cube10.stl id2 extruder 1 / id4 extruder 2) with
  skirt_type=perobject, min_skirt_length=0, brim_type=no_brim,
  combine_brims=0, flush_into_support=0. Native groups the objects with
  the tower obstacle (Print.cpp:2821-2823/:2865-2899) and emits
  per-group skirts (GCode.cpp:4465-4485); Ares rejects
  `skirt_type per-object` — RED, must FAIL before GREEN.
- per-object-tower-control.3mf  sha256=85aabacdd30857f74fc65c6d9580d6b6fcad1423c18141a05b0b7ebdce91d3ed
  same input with skirt_type=combined: native return_code 0 and raw
  normalized diff=0 against Ares (verified CPU7,8 before the RED);
  must PASS.

## Tool-change layer-boundary retract+wipe timing (GCode.cpp:5692-5697)

- toolchange-wipe-flush.3mf  sha256=aedd1cd9b356e6693299aba0cbff4ab4a55434dc7429947b74f56c2ade258e9d
  R1 base with combined skirt and flush_into_infill=1: the wiping pass
  ends the layer with a tool transition that clears the positioned
  state, so the Ares change-layer retract arming
  (motion.rs end_layer_for_timelapse positioned gate) never fires and
  the retract+wipe falls to the first post-purge travel. Native emits
  it at the layer head after G92 E0 (change_layer :5692-5697). First
  divergence at normalized N:4933. RED, must FAIL before GREEN.
- toolchange-wipe-control.3mf  sha256=85aabacdd30857f74fc65c6d9580d6b6fcad1423c18141a05b0b7ebdce91d3ed
  same input without flush_into_*: no wiping pass, native/Ares
  byte-identical (raw normalized diff=0, CPU7,8); must PASS.

## Per-object skirt x flush_into_infill (Print.cpp:2784-2996; GCode.cpp:5330-5333)

- skirt-flush-infill.3mf  sha256=b029a0f96daf20f2e4a5f677a02af8f82e606da5affd584608631304951ccefc
  R1 base with skirt_type=perobject and flush_into_infill=1 (prime tower
  on): Native extrudes Sparse infill inside the wiping windows (76 of
  them) and groups per-object skirts around the tower obstacle; Ares
  rejects the input via the flush_into_infill guard disjunct. RED, must
  FAIL before the GREEN retires that disjunct. Guard-off probe:
  normalized diff=0 (CPU7,8). flush_into_support and flush_into_objects
  stay guarded — separate residuals (302-line support skirt placement;
  tool-change/print ordering).

## Wiping-pass two-loop order x flush_into_support (GCode.cpp:5318-5360)

- skirt-flush-support.3mf  sha256=2ac30225e66793852adc01d9abd9dfe6e0c068e09311b8632f4c050b88dca85c
  R1 base with an overhang object (mesh: column + cantilever slab), the
  TiQ2 tree-support block, object relocated to y=85, prime tower on,
  skirt_type=perobject, flush_into_support=1: Native extrudes Support in
  23 wiping windows (+8 interface) and emits per-object skirts inside
  the wiping loop before the printing/stop labels, with "; PURGING
  FINISHED" opening the normal loop. Ares rejects via the
  flush_into_support guard disjunct; guard-off it still diverges 302
  normalized lines (label triple at the bucket head, Skirt blocks
  displaced). RED, must FAIL before the GREEN mirrors the two-loop
  order. flush_into_objects stays guarded (tool-change/print ordering).

## flush_into_objects single-tool collapse (ToolOrdering.cpp:660-760)

- skirt-flush-objects.3mf  sha256=7fc25540dc506a126c618d14088e3b4dbd34b7d0b94851ec908ae0c441da3ce2
  R1 base with skirt_type=perobject and flush_into_objects=1 (both
  objects): every wall/fill of the tool-2 object is overriddable
  (is_overriddable_and_mark, ToolOrdering.cpp:686-698/:718-733), so
  collect_extruders adds no extruder for it — Native collapses to a
  single tool ("; filament: 1", second used-filament entry empty, no
  toolchange, no tower) while printing both objects with per-object
  skirts. Ares rejects via the flush_into_objects guard disjunct. RED,
  must FAIL before the GREEN gates collect.rs's extruder pushes on the
  overridable marking and retires the last guard term.
