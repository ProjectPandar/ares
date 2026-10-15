# btPainted ears fixtures (sha manifest)

- painted-w5.3mf  sha256=b29dcedcd68bb564677a7f5f4b560f12c9b54039a6f8ba22eaeac09a75c203e7
  canon base + wedge/shelf mesh, brim_type=painted brim_width=5, plus
  Metadata/brim_ear_points.txt ("brim_points_format_version=0" +
  "object_id=1|-5.0 -5.0 -7.5 5.0"). Painted ear carrier per
  bbs_3mf.cpp:217/:2962-3010 (import) and :7583-7594 (export format);
  consumed by Brim.cpp:354-412 make_brim_ears via :537/:551 with the
  btPainted gates at :456-458.
- painted-ctl-w5.3mf  sha256=a7944b776027148590b4931baa6c02aca65dc3ae70f0457851a5b8298d026832
  same input with brim_ear_points.txt removed (same-input control):
  native and ares both emit no brim; must PASS.
