# Read-only actual-conversion + native source-kernel oracle, not full Viewer.
# Same source/nixpkgs variables as procgcode.nix. Kernel extraction is checked
# separately by: node tools/chain-probe/extract-actual-speed-profile.mjs --check
(import ./procgcode.nix).overrideAttrs (old: {
  pname = "orca-slicer-actual-speed-profile";
  patches = old.patches ++ [ ./actual-speed-profile.patch ];
  postPatch = (old.postPatch or "") + ''
    cp ${./actual-speed-profile-oracle.hpp} src/actual-speed-profile-oracle.hpp
    mkdir -p src/actual-speed-profile
    cp ${./actual-speed-profile/kernel.hpp} src/actual-speed-profile/kernel.hpp
  '';
})
