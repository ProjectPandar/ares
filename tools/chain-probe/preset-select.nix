# Test-only preset collection observer at 40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19.
# When ORCA_PRESET_SELECT_BASE_RECIPE and ORCA_PRESET_SELECT_SOURCE are unset,
# builds directly on tools/chain-probe/orca-release (using ORCA_PROCGCODE_NIXPKGS).
let
  source = builtins.getEnv "ORCA_PRESET_SELECT_SOURCE";
  recipe = builtins.getEnv "ORCA_PRESET_SELECT_BASE_RECIPE";
in
if recipe == "" then
  import ./orca-release {
    pname = "orca-preset-select-40ecc66f7";
    patches = [ ./preset-select.patch ];
  }
else
  let
    base = import (builtins.toPath recipe);
  in
  assert source != "";
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
