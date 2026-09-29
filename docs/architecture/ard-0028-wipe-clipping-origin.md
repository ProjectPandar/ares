# ARD-0028: Wipe clipping uses the active upstream origin

`GCode::Wipe::wipe` clips in the coordinate frame selected by the current
generator origin, including zero-origin skirt/brim emission. Integer
truncation is not translation invariant across zero. Do not clip local
negative points and assume translating afterwards preserves behavior.

Keep Ares's existing local cursor/wipe storage for now. Record the active
emission context before travel begins, reconstruct plate coordinates at
the truncation/output boundaries when necessary, and return to model-space
semantics when a model path begins. The prior path's role does not determine
the current frame. Extruder offsets apply after geometric conversion.
