# ARD-0029: Emission origins come from integer instance shifts

Emission must use the unscaled integer `PrintInstance::shift`, not the
world-space bounding-box midpoint. Scale instance translation and the
untranslated raw model center separately with upstream truncation, add
the integer coordinates, and unscale once. Bounds helpers remain suitable
for descriptive world geometry but are not emission-origin providers.

Reuse the existing validated raw-center calculation. Retain local path
storage and subtract extruder offsets after origin addition. Do not fix
boundary differences by rounding world centers or loosening G-code tests.

Only resolved printable objects may provide an emission origin; a source
file may contain unvalidated nonprinting objects before them. Translation
comes from the first printable instance, matching the upstream printable
filter. Expanding multiple printable instances remains outside this slice.
