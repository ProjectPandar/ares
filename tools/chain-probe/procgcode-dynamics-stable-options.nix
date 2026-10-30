# Pair move dynamics with OptionsZCorrector's stable marker identity.
(import ./procgcode-dynamics.nix).overrideAttrs (old: {
  pname = "orca-slicer-procgcode-dynamics-stable-options";
  patches = old.patches ++ [ ./procgcode-stable-options.patch ];
})
