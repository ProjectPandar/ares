# CLI modifiers for arranging saved 3MF projects

Source pin: OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Option | Declaration in `src/libslic3r/PrintConfig.cpp` | Direct saved-project arrangement |
| --- | --- | --- |
| `--allow-rotations` | `:13202–13205` | Defaults off; permits packing rotations. |
| `--align-to-y-axis` | `:13207–13211` | Only an explicit value overrides the loaded printer's default: on for i3, off otherwise. |
| `--allow-multicolor-oneplate` | `:13197–13200` | Defaults on; when false the arrangement restricts which material groups can share a plate. |
| `--avoid-extrusion-cali-region` | `:13213–13216` | Defaults off; accepted but does not add a keep-out on this path. |

`Config.cpp::DynamicConfig::read_cli` (`:1754–1781,1813–1855,1865–1871,1900–1906`) accepts a bare boolean flag as true and an explicit `=VALUE` using the case-insensitive CLI boolean spellings. Repeated scalar values use the last occurrence. Do not introduce a `--no-…` alias. The modifiers are not arrange actions; `--arrange` retains its existing eligibility and scope rules.

`OrcaSlicer.cpp:1597–1603,1624–1626,1670` reads the options. The direct arrange branch (`:5957–5992`) supplies the values to `libslic3r::arrangement::ArrangeParams` (`Arrange.hpp:141–149`) and updates the polygons before packing. `Arrange.cpp:249–270,396–406,823–843,1178–1194` owns axis alignment, packing rotation, and material grouping. Resolve the options into the existing core `ArrangeSettings` at the CLI arrange-job boundary using the loaded printer settings; do not run the GUI popup's normalization or change GUI defaults. The CLI only parses arguments and forwards the core action inputs. `OrcaSlicer.cpp:6011–6013,6562–6573` adds the no-rotation hint only when a packing pass ran without rotations.

The direct 3MF branch assigns the calibration parameter but does not construct the extrusion-calibration obstacle; its existing bed exclusions are independent of this flag (`OrcaSlicer.cpp:5957–5992`). Only the assemble-list arrangement branch constructs the obstacle (`OrcaSlicer.cpp:5375,5484–5486`; `PartPlate.cpp:7791–7820`). Retain the direct-project no-op; `--load-assemble-list` and its keep-out behavior are deferred. No new STL arrangement or packer.

## Integration and verification

`ares-cli/src/args.rs` and `ares-cli/src/main.rs` overlap with the saved-project X/Y rotation and grounding, standalone settings export, and selected-plate repetitions changes. Preserve their arguments, action dispatch and transform ordering when combining them; retain the Native plate-selection and repetitions behavior. The modifiers by themselves do not select those actions.

Compare complete Native and Ares G-code and Linux `result.json` (only the existing generator and wall-clock normalization), status, and output presence in both Native modes. Verify rotation admits an object that otherwise does not fit, explicit false overrides an omitted i3 Y-alignment setting, and disabling multicolor changes selected-plate admission for distinct material groups. For the calibration option separately require unchanged whole-file G-code within each executable when toggling the flag on a direct saved-project arrange.
