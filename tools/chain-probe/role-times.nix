# Read-only actual-conversion + native source-kernel oracle, not full Viewer.
# Same source/nixpkgs variables as procgcode.nix. Kernel extraction is checked
# separately by: node tools/chain-probe/extract-role-times.mjs --check
(import ./procgcode.nix).overrideAttrs (old: {
  pname = "orca-slicer-role-times";
  patches = old.patches ++ [ ./role-times.patch ];
  postPatch = (old.postPatch or "") + ''
    cp ${./role-times-oracle.hpp} src/role-times-oracle.hpp
    mkdir -p src/role-times
    cp ${./role-times/kernel.hpp} src/role-times/kernel.hpp
  '';
})
