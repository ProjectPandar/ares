# The approved marker-only diagnostic paired with the same Time source kernel.
(import ./marker-time.nix).overrideAttrs (old: {
  pname = "orca-slicer-marker-time-stable-options";
  patches = old.patches ++ [ ./procgcode-stable-options.patch ];
})
