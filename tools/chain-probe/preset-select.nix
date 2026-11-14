# Test-only preset collection observer at 40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19.
# Supply the checked 2.5.0 source and its existing dependency/build recipe.
# ORCA_PRESET_SELECT_BASE_RECIPE=/absolute/path/to/2.5.0-build.nix
# ORCA_PRESET_SELECT_SOURCE=/absolute/path/to/40ecc66f7-source
# nix build --impure --cores 4 --max-jobs 1 --file tools/chain-probe/preset-select.nix \
#   --out-link /absolute/path/to/gcroots/preset-select-40ecc66f7
let
  source = builtins.getEnv "ORCA_PRESET_SELECT_SOURCE";
  recipe = builtins.getEnv "ORCA_PRESET_SELECT_BASE_RECIPE";
  base = import (builtins.toPath recipe);
in
assert source != "";
assert recipe != "";
assert base.version == "2.5.0";
base.overrideAttrs (old: {
  pname = "orca-preset-select-40ecc66f7";
  src = builtins.path {
    path = builtins.toPath source;
    name = "orca-preset-select-40ecc66f7-source";
    filter = path: type: builtins.baseNameOf path != ".git";
  };
  patches = (old.patches or []) ++ [ ./preset-select.patch ];
})
