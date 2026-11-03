# Static one-extruder filament maps

These authored 3MF inputs derive from
`../m140-initial-bed/nonzero-initial-tool.3mf`
(`44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a`).
Only `Metadata/project_settings.config` changes; geometry, tool assignments,
logical slot order and all other archive entries remain unchanged.
No Native-exported settings are input material.

- `high-flow.3mf`: `nozzle_volume_type = ["High Flow"]`;
  SHA256 `6c61b5ca2432a4cd8793fe7c8288b85243a7c303310b5328634337ac0e302a06`.
  Tool 1 uses High Flow; the unused slot keeps Standard, yielding `0,1`.
- `unused-assignment.3mf`: `filament_volume_map = ["1","1"]`;
  SHA256 `1f001db69a4803b6f6f37f12d79dea42a23d156c4fe089e10bada97492089768`.
  The correctly sized unused assignment remains High Flow; used tool 1
  becomes Standard, yielding `1,0`.

The owning source is ToolOrdering.cpp:1752–1776,1892–1908,3054–3084,
MultiNozzleUtils.cpp:316–369, FilamentGroupUtils.cpp:346–358 and
Print.cpp:4656–4745 at `345efec64c7b95a05ce5cf96df0dde66a158835c`.
The public comparisons retain the strict generator-only whole-output oracle.
