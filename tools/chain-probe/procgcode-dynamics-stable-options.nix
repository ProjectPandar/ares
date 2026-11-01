# Beta OptionsZCorrector::on_moves_inserted owns the correction upstream.
# Keep a distinct derivation identity; do not stack the retired diagnostic fix.
(import ./procgcode-dynamics.nix).overrideAttrs (_: {
  pname = "orca-slicer-procgcode-dynamics-stable-options";
})
