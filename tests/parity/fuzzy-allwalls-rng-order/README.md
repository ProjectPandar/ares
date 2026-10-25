# fuzzy-allwalls-rng-order fixtures

Authored single-key witnesses derived from the Creality K1 (0.4 nozzle) stock
system presets (`machine.json` / `process.json` / `filament.json` flattened in
`/home/indexyz/ares-evidence/w6-k1-voron-discovery-20261024T223329/flatten-Creality/`)
and `OrcaSlicer/tests/data/overhang.obj`, exported through the official
OrcaSlicer 2.4.2 CLI `--export-3mf` surface on CPU17. These are authored
discovery inputs (single-key deltas from the stock export), not factory preset
bundles; per-key delta proof:
`/home/indexyz/ares-evidence/w4-fuzzy-surface-discovery/effective-delta-proof.json`.

| file | single-key delta vs stock | SHA256 |
| --- | --- | --- |
| `stock.3mf` | none (W6-qualified stock export) | `5d4a5f95fd4c12244e87b17e9e64052f56d66fd83df405438fb4c4e34c8c524c` |
| `external.3mf` | `fuzzy_skin="external"` | `13533832531bfd43357a6651788bbecc64f0e69494252eeca1274194c4bc932b` |
| `all.3mf` | `fuzzy_skin="all"` | `7b7f1475e6e7cb8c9f60f6f1446708edd03544e358af59147087e322eeb7fe4e` |
| `allwalls.3mf` | `fuzzy_skin="allwalls"` | `d455419d2782f950d4ffab379a973ca7ba16f1cbf7df871796a3d44b9eefe0e7` |

Shared frozen settings (`Metadata/project_settings.config`):
`wall_generator="classic"`, `fuzzy_skin_thickness="0.3"`,
`fuzzy_skin_point_distance="0.8"`, `fuzzy_skin_first_layer="0"`,
`fuzzy_skin_mode="displacement"`, `fuzzy_skin_noise_type="classic"`.

The seeded Native oracle (`orca-slicer-seed-replay-2.4.2`, `ORCA_REPLAY_SEED=0`)
is deterministic for these inputs; the official oracle is not (see
`docs/architecture/2026-10-25-fuzzy-allwalls-rng-order-spec.md`).
