# Explicitly corrected diagnostic oracle; never replaces procgcode.nix.
# Same ORCA_PROCGCODE_SOURCE / ORCA_PROCGCODE_NIXPKGS environment as that recipe.
# The only extra patch preserves OptionsZCorrector's marker index across
# actual-speed insertions. Keep both executable hashes in comparison artifacts.
(import ./procgcode.nix).overrideAttrs (old: {
  pname = "orca-slicer-procgcode-stable-options";
  patches = old.patches ++ [ ./procgcode-stable-options.patch ];
})
