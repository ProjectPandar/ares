import { useState } from "react";
import {
  activeFilamentNames,
  assignFilament,
  filamentPalette,
  type FilamentSelection,
} from "../state/filamentAssignment";
import { useApp } from "../state/store";

import "./filamentAssignment.css";

export function FilamentAssignment({
  targets,
  onClose,
}: {
  targets: readonly FilamentSelection[];
  onClose: () => void;
}) {
  const [open, setOpen] = useState(false);
  const scene = useApp((state) => state.scene);
  const selection = useApp((state) => state.presetSelection);
  const available = useApp((state) => Boolean(state.plater) && !state.arranging && state.slice.status !== "slicing");
  const settings = scene?.settings ?? null;
  const names = activeFilamentNames(selection, scene);
  const colours = filamentPalette(selection, settings, scene?.filamentSlots);
  if (!available || names.length < 2) return null;
  if (targets.every(({ instance, object }) => instance !== undefined && object === targets[0].object)) return null;
  const volumes = targets.map(({ object, volume }) =>
    scene?.objects.find(({ id }) => id === object)?.volumes.find(({ id }) => id === volume),
  );
  const modifier = volumes.some((volume) => volume?.type === "ParameterModifier");
  if (
    !targets.some(
      ({ volume }, i) =>
        volume === undefined || volumes[i]?.type === "ModelPart" || volumes[i]?.type === "ParameterModifier",
    )
  )
    return null;
  const label = targets.length === 1 ? "Change Filament" : "Set Filament for selected items";
  const choose = (slot: number) => {
    onClose();
    void assignFilament({ targets }, slot);
  };
  return (
    <div className="assign-filament" role="presentation">
      <button
        type="button"
        role="menuitem"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        onKeyDown={(event) => {
          if (event.key === "ArrowRight") {
            event.preventDefault();
            setOpen(true);
          }
        }}
      >
        {label}
        <span aria-hidden="true">›</span>
      </button>
      {open && (
        <div className="assign-filament-choices" role="menu" aria-label={label}>
          {modifier && (
            <button type="button" role="menuitem" onClick={() => choose(0)}>
              Default
            </button>
          )}
          {names.map((name, index) => (
            <button
              key={index}
              type="button"
              role="menuitem"
              onClick={() => choose(index + 1)}
              aria-label={name ? `Filament ${index + 1}: ${name}` : `Filament ${index + 1}`}
            >
              <span className="filament-swatch" aria-hidden="true" style={{ background: colours[index] ?? "#888" }}>
                {index + 1}
              </span>
              {name || `Filament ${index + 1}`}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
