# Native CLI retained-preset refresh inputs

Source: OrcaSlicer v2.5.0 `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
`libslic3r/PrintConfig.cpp:12793–12797,13096–13106` defines the switch and
semicolon-separated lists. `OrcaSlicer.cpp:1374–1375,1581–1583` reads them;
`:1849–1893` resolves the retained system identities from the saved project.

- `--uptodate` is a Native switch, not a requirement to supply
  `--project-preset-catalog`. The latter remains a separate Ares input.
  Without the switch or without a saved configuration, the two lists do not
  substitute presets into the project.
- `OrcaSlicer.cpp:2539–2624` reads flat machine/process files in list order,
  checks their kinds and retained system names, and does not replace an
  explicitly selected machine/process. A nonempty list replaces the initial
  resource-file lookup, not the later retained-system resolver.
- `:2626–2690,2692–2768,2793–2817,3164–3200` first looks for the named flat
  files in the resource directory, then resolves retained system presets
  when applicable. Missing resources do not invent a replacement. Explicit
  filament replacement takes precedence over filament refresh files. Otherwise,
  an explicit list must have one file per project filament, with the right
  type and system name at each position. The count failure is invalid argv;
  type and identity failures are invalid preset files.
- The CLI reads the files; core receives only supplied bytes/configuration.
  The flattened records supply retained values to `profiles/project_refresh`
  rather than to replacement selection. Keep the existing retained merge,
  dirty-key protection, and defaults/override ordering; the corresponding
  Native application branches are `:3213–3253,3345–3403,3510–3692`.

The witness modifies saved process, machine and filament values and supplies
matching named refresh inputs with different values. Compare the entire
sliced output using the existing Native replay and whole-file comparison.
Compare count, name, and type rejections at the CLI, including the public
result report. Existing catalog-bearing and unresolved-project paths remain
independent controls.

This slice does not port the independent printer-size admission checks
(`:2579–2599`), new compatibility rules, profile normalization rules,
packaging, or GUI/project-preset selection behavior.
