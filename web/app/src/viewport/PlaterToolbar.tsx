import { type ReactNode, useRef, useState } from "react";
import { OrcaIcon } from "../shell/icons";
import { ctrlLabel } from "../shell/keys";
import { importFile } from "../state/actions";
import { addPlate, changeInstances, MAX_PLATES } from "../state/plates";
import { useApp } from "../state/store";
import { instanceEdits } from "../sidebar/ObjectMenu";
import { filamentNames } from "../sidebar/options";
import { ArrangeButton, ArrangePopup } from "./ArrangeMenu";
import type { GizmoMode } from "./gizmos";

/** `GLGizmosManager::init` (GLGizmosManager.cpp:243-260): the selectable gizmos' names, keys and icons. */
const GIZMOS: [name: string, key: string, icon: string][] = [
  ["Move", "M", "toolbar_move"], ["Rotate", "R", "toolbar_rotate"], ["Scale", "S", "toolbar_scale"],
  ["Lay on Face", "F", "toolbar_flatten"], ["Cut", "C", "toolbar_cut"], ["Mesh Boolean", "B", "toolbar_meshboolean"],
  ["Support Painting", "L", "toolbar_support"], ["Seam painting", "P", "toolbar_seam"],
  ["Paint-on fuzzy skin", "H", "toolbar_fuzzy_skin_paint"], ["Color Painting", "N", "mmu_segmentation"],
  ["Texture displacement", "D", "toolbar_texture_displacement"], ["Emboss", "T", "toolbar_text"],
  ["Measure", "U", "toolbar_measure"], ["Assemble", "Y", "toolbar_assembly"], ["Brim Ears", "E", "toolbar_brimears"],
];

/** The gizmos this app opens. */
const MODES: Record<string, GizmoMode> = { Move: "move", Rotate: "rotate", Scale: "scale" };

function Item({ name, title, icon, enabled, pressed, onClick }: {
  name: string;
  title: string;
  icon: string;
  enabled: boolean;
  pressed?: boolean;
  onClick?: () => void;
}) {
  return (
    <button type="button" className="toolbar-item" aria-label={name} title={title} aria-pressed={pressed} disabled={!enabled} onClick={onClick}>
      <OrcaIcon name={icon} size={24} />
    </button>
  );
}

function Bar({ name, children }: { name: string; children: ReactNode }) {
  return <div className="toolbar-group" role="toolbar" aria-label={name}>{children}</div>;
}

/**
 * The Prepare canvas's overlay toolbars side by side at its top centre (`_render_overlay_toolbars`,
 * GLCanvas3D.cpp:10150-10165): the main toolbar (`_init_main_toolbar`, :7491-7664), the gizmo
 * toolbar (GLGizmosManager) and the assembly view toolbar (`_init_assemble_view_toolbar`,
 * :7730-7778). Items without a port show disabled with their tooltip.
 */
export function PlaterToolbar({ gizmo, onGizmo }: { gizmo: GizmoMode | null; onGizmo: (mode: GizmoMode | null) => void }) {
  const theme = useApp((state) => state.theme);
  const scene = useApp((state) => state.scene);
  const selected = useApp((state) => state.selectedObject);
  const slicing = useApp((state) => state.slice.status === "slicing");
  const [arranging, setArranging] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  // GLToolbar::render picks the `_dark` icons in the dark theme; Texture displacement has one.
  const icon = (name: string) => (theme === "dark" && name !== "toolbar_texture_displacement" ? `${name}_dark` : name);
  const index = scene?.objects.findIndex(({ id }) => id === selected) ?? -1;
  const edits = instanceEdits(scene?.objects[index]);
  const plates = scene?.plates.length ?? 0;
  const gizmos = filamentNames(scene?.settings ?? null).length > 1 ? GIZMOS : GIZMOS.filter(([name]) => name !== "Color Painting");
  return (
    <>
    <div className="plater-toolbar" data-testid="plater-toolbar">
      <Bar name="Main toolbar">
        <Item name="Add" title={`Add [${ctrlLabel}I]`} icon={icon("toolbar_open")} enabled={!slicing} onClick={() => input.current?.click()} />
        <input ref={input} type="file" accept=".3mf,.stl" hidden
          onChange={(event) => {
            const picked = event.target.files?.[0];
            event.target.value = "";
            if (picked) void importFile(picked);
          }} />
        <Item name="Add plate" title="Add plate" icon={icon("toolbar_add_plate")} enabled={!slicing && plates < MAX_PLATES}
          onClick={() => void addPlate()} />
        <Item name="Auto orient" icon={icon("toolbar_orient")} enabled={false}
          title={"Auto orient all/selected objects [Q]\nAuto orient all objects on current plate [Shift+Q]"} />
        <ArrangeButton open={arranging} onToggle={setArranging} icon={icon("toolbar_arrange")} />
        <span className="toolbar-separator" />
        <Item name="Add instance" title="Add instance [+]" icon={icon("instance_add")} enabled={!slicing && edits.add}
          onClick={() => void changeInstances(index, true)} />
        <Item name="Remove instance" title="Remove instance [-]" icon={icon("instance_remove")} enabled={!slicing && edits.remove}
          onClick={() => void changeInstances(index, false)} />
        <Item name="Split to objects" title="Split to objects" icon={icon("split_objects")} enabled={false} />
        <Item name="Split to parts" title="Split to parts" icon={icon("split_parts")} enabled={false} />
        <Item name="Variable layer height" title="Variable layer height" icon={icon("toolbar_variable_layer_height")} enabled={false} />
      </Bar>
      <span className="toolbar-separator" />
      <Bar name="Gizmos">
        {gizmos.map(([name, key, file]) => {
          const mode = MODES[name];
          // GLGizmoMove3D / Rotate3D / Scale3D::on_get_name without a selection.
          const title = mode && index < 0 ? `${name}:\nPlease select at least one object. [${key}]` : `${name} [${key}]`;
          return (
            <Item key={name} name={name} title={title} icon={icon(file)} enabled={!!mode && index >= 0} pressed={gizmo === mode && !!mode}
              onClick={() => onGizmo(gizmo === mode ? null : mode)} />
          );
        })}
      </Bar>
      <span className="toolbar-separator" />
      <Bar name="Assembly view">
        <Item name="Assembly View" title="Assembly View" icon={icon("toolbar_assemble")} enabled={false} />
      </Bar>
    </div>
    {arranging && <ArrangePopup />}
    </>
  );
}
