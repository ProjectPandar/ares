# ARD: sequential origin hop uses the writer frame

The upstream owner is `GCode.cpp::travel_to:7480–7506`, with conversion in `point_to_gcode:8124–8128` and local-frame maintenance in `set_origin:5659–5669`, pinned to `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

The Rust rewrite destination is the existing sequential transition caller. It must reuse `state.offset` for the hop's emitted and cached XY values. This is a narrow upstream port slice, not a new Ares pipeline. Generic writer, processor, M73, template, timing, label and lift boundaries are not in scope.

The RED target is the stock-offset P1S ByObject whole stream with the unchanged complete comparator. The same-scene zero-offset fixture and existing A1 ByObject fixture are controls. The stock target's M73 relocation is retained in the target and may be a consequence of the 2mm hop's timing; acceptance nevertheless requires complete parity. If the hop-only GREEN leaves any M73 difference, STOP rather than broadening or weakening the slice.
