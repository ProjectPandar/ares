# btPainted ears fixtures (sha manifest)

- painted.3mf  sha256=b29dcedcd68bb564677a7f5f4b560f12c9b54039a6f8ba22eaeac09a75c203e7
  canon base + wedge/shelf mesh, brim_type=painted brim_width=5, plus
  Metadata/brim_ear_points.txt ("brim_points_format_version=0" +
  "object_id=1|-5.0 -5.0 -7.5 5.0"). Painted ear carrier per
  bbs_3mf.cpp:217/:2962-3010 (import) and :7583-7594 (export format);
  consumed by Brim.cpp:354-412 make_brim_ears via :537/:551 with the
  btPainted gates at :456-458.
- painted-control.3mf  sha256=a7944b776027148590b4931baa6c02aca65dc3ae70f0457851a5b8298d026832
  same input with brim_ear_points.txt removed (same-input control):
  native and ares both emit no brim; must PASS.

## btPainted inner call site (Brim.cpp:551-554)

- painted-inner.3mf  sha256=6adadabe73cd91d8c87ced07d952dd6bc5c8458e404cd1de43c80c8ad75b332e
  pocket mesh (hole open to the bed, sealed at z=3.5mm so normal(auto)
  support fills the cavity; Brim.cpp:590-634 mask) with brim_type
  painted, brim_width 5 and a brim_ear_points.txt hole point
  (0,0,-7.5,r5). Native clips the in-hole painted disc (:551-554) to
  a 4-E-move sliver; the missing inner path emits none.
- painted-inner-control.3mf  sha256=bdb02a48d4da6c24f8a622836356499347d1166dee0611a265925ce787801bdd
  same input without brim_ear_points.txt (control): both engines emit
  no brim and must match byte-for-byte.

## btPainted EFC snap (Brim.cpp:392-411)

- painted-efc.3mf  sha256=c92ffdef749533af50bdb063c9d7d838f8802460f4564260f5985297c9750a68
  painted-inner fixture base with brim_use_efc_outline=1 and
  elefant_foot_compensation 0.1 / 1 layer / no raft (gate :56-62).
  Native snaps the painted ear centre to the closest vertex of the
  EFC-adjusted bottom outline (:392-411, vertex-level via
  closest_point_on_matching_island :99-108); the skirt follows the
  changed brim hull. Ares (pre-GREEN) does not snap: RED.
- painted-efc-control.3mf  sha256=03660965077394763c5841a1e3a78c9e3e11b2833fb902a5e55b20c0c62c837a
  same input with brim_use_efc_outline=0 (gate closed): native and ares
  stay byte-identical; must PASS.
