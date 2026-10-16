# Read-only actual-conversion + native source-kernel oracle, not full Viewer.
# Same source/nixpkgs variables as procgcode.nix. Kernel extraction is checked
# separately by: node tools/chain-probe/extract-segment-colors.mjs --check
(import ./procgcode.nix).overrideAttrs (old: {
  pname = "orca-slicer-segment-colors";
  patches = old.patches ++ [ ./segment-colors.patch ];
  postPatch = (old.postPatch or "") + ''
    cp ${./segment-colors-oracle.hpp} src/segment-colors-oracle.hpp
    mkdir -p src/segment-colors
    cp ${./segment-colors/kernel.hpp} src/segment-colors/kernel.hpp
  '';
})
