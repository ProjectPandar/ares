# Read-only actual-conversion + native source-kernel oracle, not full Viewer.
# Same source/nixpkgs variables as procgcode.nix. Kernel extraction is checked
# separately by: node tools/chain-probe/extract-segment-hw.mjs --check
(import ./procgcode.nix).overrideAttrs (old: {
  pname = "orca-slicer-segment-hw";
  patches = old.patches ++ [ ./segment-hw.patch ];
  postPatch = (old.postPatch or "") + ''
    cp ${./segment-hw-oracle.hpp} src/segment-hw-oracle.hpp
    mkdir -p src/segment-hw
    cp ${./segment-hw/kernel.hpp} src/segment-hw/kernel.hpp
  '';
})
