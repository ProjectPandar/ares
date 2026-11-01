# Beta OptionsZCorrector::on_moves_inserted owns the correction upstream.
# Keep a distinct derivation identity; do not stack the retired diagnostic fix.
(import ./marker-time.nix).overrideAttrs (_: {
  pname = "orca-slicer-marker-time-stable-options";
})
