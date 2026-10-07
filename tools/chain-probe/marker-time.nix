# Read-only actual-conversion + native source-kernel oracle, not full Viewer.
# Same source/nixpkgs variables as procgcode.nix. Kernel extraction is checked
# separately by: node tools/chain-probe/extract-marker-time.mjs --check
(import ./procgcode.nix).overrideAttrs (old: {
  pname = "orca-slicer-marker-time";
  patches = old.patches ++ [ ./marker-time.patch ];
  postPatch = (old.postPatch or "") + ''
    cp ${./marker-time-oracle.hpp} src/marker-time-oracle.hpp
    mkdir -p src/marker-time
    cp ${./marker-time/kernel.hpp} src/marker-time/kernel.hpp
  '';
})
