# Direct CLI `--enable-timelapse`

Upstream: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

## CLI contract

- `src/libslic3r/PrintConfig.cpp:13173–13176` defines a CLI boolean defaulting
  to false. `src/libslic3r/Config.cpp:1758–1782,1847–1855,1900–1906`
  reads the bare option as true, or parses an attached boolean value.
  `src/OrcaSlicer.cpp:1545,1593–1595` reads it. The only use of the raw
  boolean after parsing is `:4303–4306`: the CLI condition requires both
  that boolean and an effective `timelapse_type` of Smooth. The process
  mode is not changed; its default is Traditional (`PrintConfig.cpp:6928–6937`).
- `OrcaSlicer.cpp:2531–2535` enables the post-mapping tower disable when a
  saved BBL-format project maps to one loaded filament preset. The CLI
  can cancel that disable for distinct colors (`:3874–3891`), wrapping
  detection (`:4286–4297`), mixed filament (`:4309–4315`), or enabled
  Smooth timelapse (`:4316–4320`). Otherwise it sets `enable_prime_tower`
  false and records the changed key (`:4322–4342`). Smooth does **not**
  turn a disabled `enable_prime_tower` on.
- The same CLI condition admits a single-filament tower into the saved
  plate's estimated bounds (`:4540–4571,4584–4597`, called at `:4708`;
  those bounds affect recentering at `:4654–4677`). Whole-bed arrangement
  uses it to admit single-filament and by-object project candidates, still
  requiring prime tower and excluding belt printers (`:5729–5748`). The
  selected-plate arrange path uses it as a sequence exception and to avoid
  dropping a single-filament obstacle (`:5855–5888,5897–5944`).

## Print and G-code consumers

The flag itself does not reach `Print::apply` (`OrcaSlicer.cpp:6986`).
`Print::enable_timelapse_print` (`Print.cpp:5235–5238`) checks only the
mode. `Print::has_wipe_tower` (`:5189–5207`) rejects belts; with an
already-enabled prime tower, Smooth bypasses the multi-filament check.
Smooth also prevents an early stop in tower layer planning (`:5435–5442`).
Process normalization also checks the mode rather than the flag
(`PrintConfig.cpp:10404–10412,10487–10504`).

`GCode.hpp:125,154` derives the tower-integration Smooth bit from the mode,
and `GCode.cpp:6711–6723` selects farthest-point and Traditional placement
independently of the CLI boolean. The configured template is rendered at
layer change for the non-BBL branch (`:6738–6746`), or via the BBL
layer-start path (`:7407–7420`), or from the existing Traditional insertion
paths (`:7507–7559,7771–7777,8072–8078,8193–8227`). The farthest-point inline
hook is at `:9974–9996`. A flag can change emitted G-code indirectly by
preserving the tower, but it does not itself enable the template or turn
Traditional photo G-code off when absent.

| Invocation | CLI condition | One-preset mapping on a saved project, with no other exceptions | Print and G-code |
| --- | --- | --- | --- |
| Flag absent (or false) | False even for Smooth | Disables the tower | Keeps the configured mode; Smooth without a tower is not rewritten to Traditional. Traditional templates can still render. |
| Flag with Traditional | False | Disables the tower, as if absent | Keeps Traditional; no flag-specific output change. |
| Flag with Smooth | True | Preserves the original prime-tower option | Uses the existing Smooth behavior and prints the tower when the existing print conditions permit it. |

## Route discrimination and Rust boundary

- For ordinary direct STL, `OrcaSlicer.cpp:1936` populates `used_filament_set`,
  `:2042–2045` resets the arrangement plate to 0, and `:5689,5732,5741–5744`
  use the all-bed STL branch, which admits the footprint independently of the
  flag. STL also cannot satisfy the saved-project mapping condition at `:2531`.
- A Native probe used one synthetic block on a Creality Ender-3 0.4 mm
  nozzle with Smooth, an enabled prime tower, and a template printing the
  effective mode. With loaded presets and automatic STL arrangement, both
  flag settings exited successfully and had identical G-code after generator
  banner normalization, with 50 Smooth template lines and the tower enabled.
  Both fixed-position STL trials rejected the placement. The same block
  saved as a 3MF and sliced with `--arrange 0` and no remap gave identical
  successful output, retaining the tower whether or not the flag was set.
  Reloading one filament preset on that saved project kept both exits and
  the 50 template lines unchanged, but changed `enable_prime_tower` from 0
  without the flag to 1 with it. Verifying the flag on that remap seam
  requires direct saved-project preset loading.

Map the native-style direct argument to the runtime boolean for
`project/domain/profile_input/mapped_prime_tower.rs`
(`OrcaSlicer.cpp:1593–1595,4303–4306`), not a raw project key. Reuse the
existing selected-profile path and its post-mapping tower logic.
Do not represent the flag as a project option, change the process mode,
or add Print, G-code, or wipe-tower generation code.

Use one synthetic, one-extruder, by-layer saved 3MF with a loaded filament
preset and a template that identifies the process mode. Compare Smooth
without the flag to Smooth with the flag, with Traditional with the flag
as the default-mode control. Retain the saved Smooth case without a remap
as a negative control. Run Native and Ares with identical arguments
and private temporary directories. Compare whole G-code, exit, stderr,
result JSON and recursive working-directory inventory, using only the
existing volatile metadata normalization. Saved-project arrange and bed
migration, multi-plate and mixed-filament cases are outside this slice.
