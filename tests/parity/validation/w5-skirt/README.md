# Per-object skirt x wipe tower RED fixtures (W5)

- per-object-tower-w5.3mf  sha256=d9ee50b057550861a5bcf7ef9fa62a90e08c54cb3c197ebf5255b6c4122e145b
  Derived from tests/parity/semm-ramming/R1.3mf
  (sha256=0b765abbf5dcc3d7196a5555dba9241fc6c5d55830e0b986cf9fbc9a62b00b9f,
  Creality Ender-3 SEMM, single nozzle, two filaments, prime tower
  enabled, objects cube10.stl id2 extruder 1 / id4 extruder 2) with
  skirt_type=perobject, min_skirt_length=0, brim_type=no_brim,
  combine_brims=0, flush_into_support=0. Native groups the objects with
  the tower obstacle (Print.cpp:2821-2823/:2865-2899) and emits
  per-group skirts (GCode.cpp:4465-4485); Ares rejects
  `skirt_type per-object` — RED, must FAIL before GREEN.
- per-object-tower-ctl-w5.3mf  sha256=85aabacdd30857f74fc65c6d9580d6b6fcad1423c18141a05b0b7ebdce91d3ed
  same input with skirt_type=combined: native return_code 0 and raw
  normalized diff=0 against Ares (verified CPU7,8 before the RED);
  must PASS.

## Tool-change layer-boundary retract+wipe timing (GCode.cpp:5692-5697)

- toolchange-wipe-flush-w5.3mf  sha256=aedd1cd9b356e6693299aba0cbff4ab4a55434dc7429947b74f56c2ade258e9d
  R1 base with combined skirt and flush_into_infill=1: the wiping pass
  ends the layer with a tool transition that clears the positioned
  state, so the Ares change-layer retract arming
  (motion.rs end_layer_for_timelapse positioned gate) never fires and
  the retract+wipe falls to the first post-purge travel. Native emits
  it at the layer head after G92 E0 (change_layer :5692-5697). First
  divergence at normalized N:4933. RED, must FAIL before GREEN.
- toolchange-wipe-ctl-w5.3mf  sha256=85aabacdd30857f74fc65c6d9580d6b6fcad1423c18141a05b0b7ebdce91d3ed
  same input without flush_into_*: no wiping pass, native/Ares
  byte-identical (raw normalized diff=0, CPU7,8); must PASS.
