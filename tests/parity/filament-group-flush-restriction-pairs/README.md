# FlushMode height and printable-area assignment pairs

These inputs exercise `PrintObject.cpp:349–515`, `Print.cpp:3593–3604` and
`ToolOrdering.cpp:1537–1672`: a restriction on the second physical head changes
the assignment of the middle logical filament without making the first head
unable to print any model.

## Derivation

`x2d_pla_both_heads_reach_models.3mf` is a byte-identical copy of
`../filament-group-flush-context/x2d_pla_three_objects_default_purge.3mf`.
The other two archives are derived independently from that control. Preserve
every ZIP entry's name, order and metadata. Decode only
`Metadata/project_settings.config`, replace the single setting listed below,
and serialize that JSON with sorted keys and compact separators. Copy all other
entry payloads unchanged. Do not change models, transforms, sequences, matrix,
seed maps or presets.

| Restricted input | Sole changed setting | Value |
| --- | --- | --- |
| `x2d_pla_second_head_below_models.3mf` | `extruder_printable_height` | `["261", "0.1"]` |
| `x2d_pla_second_head_area_outside_models.3mf` | `extruder_printable_area` | `["0x0,256x0,256x256,0x256", "0x0,1x0,1x1,0x1"]` |

Both pairs use the same control. The heights change only the second head's
limit; the area changes only its printable polygon. The models are reachable
by both heads in the control. The first head remains unrestricted in both
derived inputs. All three logical filaments are used.

## Frozen inputs

| Input | SHA-256 | Native physical-head map |
| --- | --- | --- |
| `x2d_pla_both_heads_reach_models.3mf` | `717250cd867278868a88c646f9754f7761dd73f1f8d461e965b294545cf3db66` | `1,2,1` |
| `x2d_pla_second_head_below_models.3mf` | `6d550d5c4a39d4ae89c98881f183d079c5dc007a1f342648f17fdde57c3d4909` | `1,1,1` |
| `x2d_pla_second_head_area_outside_models.3mf` | `2fd4454175ebf8158a258f504d54308cd8cae9bea4de5486870eb64f147a0314` | `1,1,1` |

The Native reference is OrcaSlicer 2.5.0-beta at source revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. Public tests qualify both endpoints
independently with Native replay, require successful API and CLI slicing and
complete API/CLI byte equality, then require each endpoint's physical map to
match Native and the per-filament assignment change to have the same direction.
Whole-output Native differences remain diagnostics, not whole-output parity.
Only inputs and this derivation description belong in this directory.
