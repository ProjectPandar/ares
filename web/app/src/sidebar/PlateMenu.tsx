import { useEffect, useRef } from "react";
import { deletePlateObjects } from "../state/plates";
import { useApp } from "../state/store";

/**
 * The object list's plate menu (MenuFactory::create_plate_menu, GUI_Factories.cpp:1958), as far as
 * ported: "Delete All" (:1980-1987). Closes on a pick, Escape or a click elsewhere.
 */
export function PlateMenu({ plate, position, onClose }: { plate: number; position: [number, number]; onClose: () => void }) {
  const menu = useRef<HTMLDivElement>(null);
  // `!plate->get_objects().empty()`: PartPlate::get_objects is the model's objects (PartPlate.hpp:477).
  const hasObjects = useApp((state) => (state.scene?.objects.length ?? 0) > 0);
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
    <div ref={menu} className="context-menu" role="menu" aria-label="Plate menu" style={{ left: position[0], top: position[1] }}>
      <button type="button" role="menuitem" title="Delete all objects on the current plate" disabled={slicing || !hasObjects}
        onClick={() => {
          onClose();
          void deletePlateObjects(plate);
        }}>
        Delete All
      </button>
    </div>
  );
}
