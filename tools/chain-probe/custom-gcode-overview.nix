# Read-only actual-conversion + native source-kernel oracle, not full Viewer.
# Same source/nixpkgs variables as procgcode.nix. Kernel extraction is checked
# separately by: node tools/chain-probe/extract-custom-gcode-overview.mjs --check
(import ./procgcode.nix).overrideAttrs (old: {
  pname = "orca-slicer-custom-gcode-overview";
  patches = old.patches ++ [ ./custom-gcode-overview.patch ];
  postPatch = (old.postPatch or "") + ''
    cp ${./custom-gcode-overview-oracle.hpp} src/custom-gcode-overview-oracle.hpp
    mkdir -p src/custom-gcode-overview
    cp ${./custom-gcode-overview/kernel.hpp} src/custom-gcode-overview/kernel.hpp
  '';
})
