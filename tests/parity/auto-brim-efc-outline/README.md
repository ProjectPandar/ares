# AutoBrim post-EFC consumer frozen inputs

Source: merged `tests/parity/auto-brim-object-speed/auto_brim-global-only.3mf`,
SHA256 `e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5`.
Keep mesh/model/component/instance and material/speed settings unchanged.
Set `brim_type=auto_brim`, `elefant_foot_compensation_layers=1`; vary only
`elefant_foot_compensation=0|0.3` and `brim_use_efc_outline=0|1`.

Exact input SHA256s are asserted by `auto_brim_efc_outline.rs`; all four files
are referenced. No Native output is committed. Native-first evidence and
reproducible stored-ZIP generator (`generate-inputs.js`, CRC32 per entry) live at:
`/home/indexyz/ares-coord/evidence/worker2-auto-efc-discovery-20261020T204057/`.
The generator copies the source archive and rewrites project-settings JSON
only. Verify the source hash, then replay into a fresh external root with
`taskset -c 13 node generate-inputs.js <root>/inputs`.

Dependency: exact-group chain ending `fb70752b9`; this RED stacks on it, not
on a falsely labeled current-main binary. Public tests reuse merged
`runner/native_api_cli.rs`: Native first, then API + actual CLI, unchanged
strict complete ordered comparisons and raw API/CLI equality. M73 is included.
One active amount0.3/flag1 target must fail; three inactive controls must pass.
