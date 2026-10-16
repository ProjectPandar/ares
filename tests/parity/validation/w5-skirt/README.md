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
