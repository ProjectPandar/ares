# Read-only actual-conversion + native source-kernel oracle at 40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19,
# not the whole Viewer. Kernel extraction is checked separately by:
# node tools/chain-probe/extract-segment-colors.mjs --check
# Supply the checked 2.5.0 source, its existing dependency/build recipe and the builder's taskset list:
# ORCA_SEGMENT_COLORS_SOURCE=/absolute/path/to/40ecc66f7-source
# ORCA_SEGMENT_COLORS_BASE_RECIPE=/absolute/path/to/2.5.0-build.nix
# ORCA_SEGMENT_COLORS_TASKSET=24-31
# nix build --impure --max-jobs 1 --file tools/chain-probe/segment-colors.nix \
#   --out-link /absolute/path/to/gcroots/orca-segment-colors-250
let
  source = builtins.getEnv "ORCA_SEGMENT_COLORS_SOURCE";
  recipe = builtins.getEnv "ORCA_SEGMENT_COLORS_BASE_RECIPE";
  taskset = builtins.getEnv "ORCA_SEGMENT_COLORS_TASKSET";
  base = import (builtins.toPath recipe);
  pinned = args: [ "-c" taskset ] ++ builtins.tail (builtins.tail args);
  # The patched tree, so the oracle's src is the source with the patches and the kernel applied.
  patched = base.stdenv.mkDerivation {
    name = "orca-slicer-2.5.0-segment-colors-patched";
    src = builtins.path {
      path = builtins.toPath source;
      name = "orca-slicer-2.5.0-source";
      filter = path: type: builtins.baseNameOf path != ".git";
    };
    patches = [ ./procgcode.patch ./segment-colors.patch ];
    patchFlags = [ "-p1" "--no-backup-if-mismatch" ];
    realBuilder = base.drvAttrs.builder;
    args = pinned base.drvAttrs.args;
    dontConfigure = true;
    dontBuild = true;
    dontFixup = true;
    installPhase = ''
      cp ${./segment-colors-oracle.hpp} src/segment-colors-oracle.hpp
      mkdir -p src/segment-colors
      cp ${./segment-colors/kernel.hpp} src/segment-colors/kernel.hpp
      cp -a . $out
    '';
  };
in
assert source != "";
assert recipe != "";
assert taskset != "";
assert base.version == "2.5.0";
base.overrideAttrs (old: {
  pname = "orca-slicer-segment-colors-250";
  src = builtins.path {
    path = patched;
    name = "orca-slicer-2.5.0-segment-colors-source";
  };
  args = pinned old.args;
})
