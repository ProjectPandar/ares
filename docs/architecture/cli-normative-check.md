# Normative checking of saved project settings

## Source boundary

In OrcaSlicer 2.5.0, `libslic3r/PrintConfig.cpp:12842–12845` registers `normative_check` as a CLI action of type `coBool`, defaulting to true. `OrcaSlicer.cpp:1577–1579` reads it before loading the inputs. `libslic3r/Format/bbs_3mf.cpp:2042–2048,2800–2809` imports `Metadata/project_settings.config` into the dynamic config. Immediately after loading a BBL 3MF, before combining it with explicit or retained presets, `OrcaSlicer.cpp:1800–1807` rejects a non-empty `ConfigOptionStrings` vector at `post_process` when normative checking is enabled. It tests the number of entries, not their text; `[]` is permitted while `[""]` is not. Rejection is `CLI_POSTPROCESS_NOT_SUPPORTED` (`-19`; Linux process exit 237), with plate index 0, `post_process is not supported under CLI.` in `result.json`, and no G-code. The action pass at `OrcaSlicer.cpp:6431–6432` performs no second check.

An explicit false skips only that inspection of the embedded settings. It does not skip format, preset, printer, or slice validation. The CLI call to run post-processing is commented out at `OrcaSlicer.cpp:7236`; no execution, filesystem access for scripts, or GUI behavior is added by this port.

## Rust destination

Add the runtime CLI option `--normative-check`, default true, with `--normative-check=0` and `--normative-check=1` as explicit values. `Config.cpp:1848–1856,1899–1906` uses an equals sign for explicit boolean values; with a separated `0`, Orca treats `0` as an input name, a tokenization quirk not reproduced here.

`ares-cli` passes the runtime choice to the existing `ares-core` CLI project-loading admission. Reuse the existing `PostProcessNotSupported` result family and the shared embedded-setting check, including the checks before explicit and retained preset overlay. Keep the boolean separate from `Print::validate`, the persisted project configuration, and per-option overrides. Omission preserves behavior of existing callers; profile, browser, and standalone core callers without a CLI override keep their established defaults. STL, output naming/layout, and unrelated checks are not changed.

## Public behavior checks

Use the same synthetic 3MF bytes for both CLIs, once with a `post_process` vector containing an empty string and once with an empty vector. Compare the default and `--normative-check=0` for each input: the first project must reject only in default mode; the second must succeed in both. Pass the same option token to both CLIs, with their existing equivalent single-plate export commands. Compare the exit code, stderr, Linux `result.json`, and exported filenames; the empty-string entry does not name a command. Exercise explicit `=1` as the default on the opt-in surface.
