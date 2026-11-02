import type { SceneObject } from "@ares-slicer/wasm";
import { useEffect, useRef } from "react";
import { changeInstances, mirrorInstance } from "../state/plates";
import { useApp } from "../state/store";

/** MenuFactory::append_menu_item_transform's mirror items (GUI_Factories.cpp:1353-1363). */
const MIRROR = [
  { label: "Along X Axis", tooltip: "Mirror along the X axis" },
  { label: "Along Y Axis", tooltip: "Mirror along the Y axis" },
  { label: "Along Z Axis", tooltip: "Mirror along the Z axis" },
] as const;

/**
 * `can_increase_instances` / `can_decrease_instances` (Plater.cpp:15097-15118): "Add instance" needs
 * every instance printable, "Remove instance" more than one instance.
 */
export function instanceEdits(object: SceneObject | undefined): { add: boolean; remove: boolean } {
  return {
    add: !!object && object.instances.every((instance) => instance.printable),
    remove: !!object && object.instances.length > 1,
  };
}

/** The instance items (GUI_Factories.cpp:2310-2315), with their shortcuts. */
const INSTANCE_ITEMS = [
  { label: "Add instance", shortcut: "+", tooltip: "Add one more instance of the selected object", more: true },
  { label: "Remove instance", shortcut: "-", tooltip: "Remove one instance of the selected object", more: false },
] as const;

/**
 * The object list's right-click menu, as far as ported: the instance items and the transform
 * submenu's "Mirror" items of the object menu (GUI_Factories.cpp:1765). They act on the object's first instance, as the object
 * panel does. Closes on a pick, Escape or a click elsewhere.
 */
export function ObjectMenu({ objectId, position, onClose }: { objectId: number; position: [number, number]; onClose: () => void }) {
  const menu = useRef<HTMLDivElement>(null);
  const object = useApp((state) => state.scene?.objects.findIndex(({ id }) => id === objectId) ?? -1);
  const edits = instanceEdits(useApp((state) => state.scene?.objects.find(({ id }) => id === objectId)));
  const slicing = useApp((state) => state.slice.status === "slicing");
  useEffect(() => {
    const close = (event: Event) => {
      if (event instanceof KeyboardEvent ? event.key === "Escape" : !menu.current?.contains(event.target as Node)) onClose();
    };
    document.addEventListener("pointerdown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("pointerdown", close);
      document.removeEventListener("keydown", close);
    };
  }, [onClose]);
  return (
    <div ref={menu} className="context-menu" role="menu" aria-label="Object menu" style={{ left: position[0], top: position[1] }}>
      {INSTANCE_ITEMS.map(({ label, shortcut, tooltip, more }) => (
        <button key={label} type="button" role="menuitem" title={tooltip} aria-keyshortcuts={shortcut}
          disabled={slicing || !(more ? edits.add : edits.remove)}
          onClick={() => {
            onClose();
            void changeInstances(object, more);
          }}>
          {label}
        </button>
      ))}
      <div className="context-menu-label" role="presentation">Mirror:</div>
      {MIRROR.map(({ label, tooltip }, axis) => (
        <button key={label} type="button" role="menuitem" title={tooltip} disabled={slicing || object < 0}
          onClick={() => {
            onClose();
            void mirrorInstance({ object, instance: 0 }, axis as 0 | 1 | 2);
          }}>
          {label}
        </button>
      ))}
    </div>
  );
}
