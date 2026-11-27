# Version admission for direct sliced 3MF export

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`)
defines the `allow_newer_file` Boolean option with default `false`
(`src/libslic3r/PrintConfig.cpp:13247–13250`). `CLI::run` reads the
option (`src/OrcaSlicer.cpp:1613–1615`) and checks the imported project's
producer version before proceeding to the slice and export actions
(`:1758–1781`). A BambuStudio/OrcaSlicer project with a newer major or
minor version is rejected unless the option is true. The option does
not bypass other admission, slicing, or export restrictions.

## Ares boundary

Accept `--allow-newer-file` on the direct `--slice … --export-3mf …`
route in `crates/ares-cli/src/args.rs`, with the same bare and explicit
Boolean forms as the `slice` subcommand. Pass the value through
`crates/ares-cli/src/export_3mf.rs` to the saved-project sliced exporter
in `crates/ares-core/src/project/export/sliced.rs`. The CLI-facing export
uses the existing `LoadedProfileProject::open_from_archive` version
check on its already opened archive, and uses the existing CLI
`FileVersionNotSupported` error mapping if that check fails. The core
export entry used by WASM keeps its current unchecked policy.

This includes admission for the direct saved-project sliced archive
route. It does not change plain G-code slicing, export without slicing,
STL export, preset discovery, or the archive writer's supported layouts.

## Verification

Generate a synthetic one-plate saved project and change its producer
version metadata to a newer version. Compare Native and Ares direct
command exit status and archive presence without the flag and with a
false value. With the enabled flag, compare the sliced archive with the
existing archive comparator (including plate G-code). Run the checks in
both Native reference lanes.
