# Web plater toolbar

## Source boundary

This slice ports OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c` GUI behaviour. Paths are
relative to `src/slic3r/GUI/`.

The Prepare canvas shows three toolbars side by side at its top, centred together
(`GLCanvas3D::_render_overlay_toolbars`, `GLCanvas3D.cpp:10150-10165`;
`get_main_toolbar_offset`, `:10339-10354`): the main toolbar, a separator line, the gizmo
toolbar and the assembly view toolbar (`_render_separator_toolbar_left/right`,
`_render_assemble_view_toolbar`, `:10927-10940, 11355-11382`).

- Main toolbar, `GLCanvas3D::_init_main_toolbar` (`:7491-7655`), in item order. Every item is
  visible (`GLToolbar.cpp:75`); `enabling_callback` greys it out.

  | Item | Icon | Tooltip (`update_shortcut_tooltips`, `:7656-7664`) | Enabled |
  |---|---|---|---|
  | `add` | `toolbar_open` | Add [Ctrl+I] | `can_add_model`: not slicing |
  | `addplate` | `toolbar_add_plate` | Add plate | `can_add_plate`: not slicing, fewer than `MAX_PLATES_COUNT` plates |
  | `orient` | `toolbar_orient` | Auto orient all/selected objects [Q] / Auto orient all objects on current plate [Shift+Q] | `can_arrange` |
  | `arrange` | `toolbar_arrange` | Arrange all objects [A] / Arrange objects on selected plates [Shift+A] | `can_arrange`: objects, worker idle |
  | separator | | | |
  | `more` | `instance_add` | Add instance [+] | `can_increase_instances` |
  | `fewer` | `instance_remove` | Remove instance [-] | `can_decrease_instances` |
  | `splitobjects` | `split_objects` | Split to objects | `can_split_to_objects` |
  | `splitvolumes` | `split_parts` | Split to parts | `can_split_to_volumes` |
  | `layersediting` | `toolbar_variable_layer_height` | Variable layer height | `can_layers_editing` |

  `arrange` toggles the arrange settings popup (`_render_arrange_menu`); `layersediting` is
  toggable.
- Gizmo toolbar, `GLGizmosManager::init` (`Gizmos/GLGizmosManager.cpp:243-260`), showing the
  `is_selectable` gizmos (`get_selectable_idxs`, `:90-109`) in this order. A tooltip is the
  gizmo's `get_name` with its shortcut (`GLGizmoBase::get_name`, `ShortcutRegistry::with_key`
  `Shortcuts.cpp:271-275`); Move, Rotate and Scale add ":\nPlease select at least one object."
  while nothing is selected.

  | Gizmo | Icon | Name | Key |
  |---|---|---|---|
  | Move | `toolbar_move` | Move | M |
  | Rotate | `toolbar_rotate` | Rotate | R |
  | Scale | `toolbar_scale` | Scale | S |
  | Flatten | `toolbar_flatten` | Lay on Face | F |
  | Cut | `toolbar_cut` | Cut | C |
  | MeshBoolean | `toolbar_meshboolean` | Mesh Boolean | B |
  | FdmSupports | `toolbar_support` | Support Painting | L |
  | Seam | `toolbar_seam` | Seam painting | P |
  | FuzzySkin | `toolbar_fuzzy_skin_paint` | Paint-on fuzzy skin | H |
  | MmSegmentation | `mmu_segmentation` | Color Painting | N |
  | TextureDisplacement | `toolbar_texture_displacement` | Texture displacement | D |
  | Emboss | `toolbar_text` | Emboss | T |
  | Measure | `toolbar_measure` | Measure | U |
  | Assembly | `toolbar_assembly` | Assemble | Y |
  | BrimEars | `toolbar_brimears` | Brim Ears | E |

  SVG and Simplify are not selectable. Color Painting is selectable only with more than one
  filament (`GLGizmoMmuSegmentation::on_is_selectable`).
- Assembly view toolbar, `_init_assemble_view_toolbar` (`GLCanvas3D.cpp:7730-7778`): one item,
  icon `toolbar_assemble`, tooltip "Assembly View", enabled by `Plater::has_assmeble_view`.
- Icons switch to their `_dark` file in the dark theme (`m_is_dark`); Texture displacement has
  one icon.

- Gizmo switching, `GLGizmosManager` (`Gizmos/GLGizmosManager.cpp`): at most one gizmo is open
  and none is open at first. A toolbar click or the gizmo's key (`open_gizmo_by_shortcut`,
  `:548-568`, which needs a selection) opens it, and the same click or key on the open gizmo
  closes it (`open_gizmo`, `:458-477`). A selection the open gizmo cannot act on closes it
  (`refresh_on_off_state`, `:430-438`), and clearing the selection closes it
  (`GLCanvas3D::deselect_all`, `GLCanvas3D.cpp:3063-3071`). A press on an object still drags it
  with or without a gizmo.
- The canvas's right click shows its menu only while no gizmo is open
  (`m_gizmos.get_current_type() == Undefined`, `GLCanvas3D.cpp:5459`); the object under the
  cursor is selected either way (`:5418-5430`).
- Scale gizmo, `GLGizmoScale3D` (`Gizmos/GLGizmoScale.cpp`):
  - `update_grabbers_data` (`:248-301`): on the selection's world bounding box, grabbers 0/1 at
    the bottom face's -X/+X edge centres, 2/3 at its -Y/+Y edge centres, 5 at the top face's
    centre (4, the bottom centre, is disabled), and the uniform grabbers 6-9 at the bottom
    corners. Axis grabbers take `AXES_COLOR`, the uniform ones `GRABBER_UNIFORM_COL`
    (`GLGizmoBase.cpp:76`); every grabber is a cube of `FixedGrabberSize` 16 screen pixels
    (`GLGizmoBase.cpp:170-172`).
  - `on_render` (`:317-371`): lines join the uniform grabbers round the bottom face, 6-7 and
    7-8 in the Y colour, 8-9 and 9-6 in the X colour.
  - `on_start_dragging` (`:200-228`) and `calc_ratio` (`:505-544`): the ratio is the pressed
    grabber's distance from the bottom centre plus the drag's projection on that direction, over
    that distance. The drag point moves on the bottom face's plane, or for grabber 5 on the
    vertical plane through the Z axis facing the pointer; a pointer ray within 5° of that plane
    gives 1, and Shift snaps the ratio to 0.05 steps.
  - `on_dragging` (`:236-246`), `do_scale_along_axis` and `do_scale_uniform` (`:442-503`): the
    axis grabbers scale their axis by the ratio, the uniform ones every axis, relative to the
    drag's start (`Selection::scale_and_translate` in world coordinates, `on_mouse`
    `:112-133`). Releasing applies `GLCanvas3D::do_scale` (`on_stop_dragging`, `:230-234`): the
    instance drops back onto the bed. The drag is one undo step.
- The user's screenshot shows the toolbar of a later Orca: it has no instance and split items and
  puts "Variable layer height" right after "Arrange". This slice follows the pinned
  `345efec64` source.

## Web destination

- `web/scripts/extract-process-layout.mjs` copies the toolbar icons from `resources/images`.
- `web/app/src/viewport/PlaterToolbar.tsx`: the three toolbars, centred at the top of the
  Prepare canvas, with the source's tooltips and enabling.
  - `add` opens a file picker for the import; `addplate` calls `addPlate`, which frames every
    plate; `arrange` is `ArrangeMenu`, its popup opening below the toolbar; `more` and `fewer`
    call `changeInstances` on the selected object.
  - Move, Rotate and Scale open and close the canvas gizmos, as do the M, R and S keys.
  - `orient`, `splitobjects`, `splitvolumes`, `layersediting`, the gizmos after Scale and the
    assembly view are disabled: the web has no auto orientation, split, layer height editing,
    flatten, cut, boolean, painting, text, measure, assembly or brim ear port. Color Painting
    shows only for a project with more than one filament.
- `web/app/src/viewport/scaleGizmo.ts`: the scale grabbers. `instanceDrag.ts` computes the ratio
  and sends each step's change of it.
- `ares-core` `PlaterSession::scale_instance` (`plater/scale.rs`) scales an instance by relative
  world factors about its box centre (`selection::scale`) and drops it onto the bed
  (`scale_and_drop`, `GLCanvas3D::do_scale`). The wasm `PlaterSession.scale` and the web worker's
  `scale` edit expose it.
  - Each drag step applies the ratio's change since the last step. Scaling about the box centre
    and dropping onto the bed equals one scale about the bottom centre in Z, so the steps compose
    to the release's single `do_scale`.
- `web/app/src/viewport/ViewportView.tsx`: the open gizmo (none at first), closed when the
  selection clears; the object menu only with no gizmo open.
- The Objects section's "Add plate" button, the web's stand-in for `addplate` until now, is
  removed.
- The camera view toolbar stays a web-only control and moves to the bottom of the Prepare canvas,
  where the removed gizmo toolbar was. The Preview tab keeps it at the top.
- Every toolbar edit takes an undo snapshot through the state action it calls; a gizmo drag takes
  one when it starts.

## Deferred

- Auto orientation (`Plater::orient`, `libslic3r/Orient.cpp`): neither `ares-core` nor the wasm
  adapter has an orientation API.
- The scale gizmo's Ctrl (one-sided) and Alt variants, its input window and hover colours.
- Escape closing the open gizmo before it clears the selection (`GLCanvas3D::on_char`).
- The right-click actions of the main toolbar items and the toolbar's collapse, scaling and
  overflow layout (`GLToolbar::set_scale`, `GLCanvas3D.cpp:8630-8640`).

## Verification

`web/e2e/platerToolbar.spec.ts` loads the two-plate project and records the toolbar screenshot
in both themes. It checks the toolbar order, icons and tooltips against the source.

- "Add plate" must add a plate and frame all plates.
- "Arrange" must open its popup, and its "Arrange" must move the objects.
- Move, Rotate and Scale must open their gizmo, and a second click or key must close it.
- An X grabber drag must scale X only, and a uniform grabber drag every axis by one ratio, each
  undone by one Ctrl+Z.
- Every unported item must be disabled.

`plateMove.spec.ts` opens the move gizmo before its grabber drag. `viewportObjectMenu.spec.ts`
checks that a right click with a gizmo open selects the object without a menu.
