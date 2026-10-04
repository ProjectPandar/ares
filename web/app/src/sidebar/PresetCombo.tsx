import { useEffect, useId, useRef, useState } from "react";

export interface PresetEntry {
  name: string;
  /** Filament presets draw their `filament_colour` (PresetComboBox::get_bmp). */
  colour?: string;
}

/**
 * PlaterPresetComboBox (PresetComboBoxes.cpp) for presets embedded in the
 * loaded project: Orca lists them under "Project presets"
 * (Preset::is_project_embedded). System and user presets need a preset bundle
 * the engine does not expose yet, so this is the only group.
 */
export function PresetCombo({ label, value, presets, placeholder }: {
  label: string;
  value?: string;
  presets: PresetEntry[];
  placeholder: string;
}) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const listId = useId();

  useEffect(() => {
    if (!open) return;
    const onPointer = (event: PointerEvent) => {
      if (!root.current!.contains(event.target as Node)) setOpen(false);
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };
    document.addEventListener("pointerdown", onPointer);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onPointer);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div className="combo-root" ref={root}>
      <button type="button" className={value ? "combo" : "combo empty"} title={value ?? placeholder} aria-label={label}
        aria-haspopup="listbox" aria-expanded={open} aria-controls={open ? listId : undefined}
        disabled={presets.length === 0} onClick={() => setOpen(!open)}>
        <span className="combo-text">{value ?? placeholder}</span>
        <span className="combo-caret" />
      </button>
      {open && (
        <div className="combo-popup" role="listbox" id={listId} aria-label={label}>
          <div role="group" aria-label="Project presets">
            <div className="combo-group-title" role="presentation">Project presets</div>
            {presets.map((preset) => (
              <div key={preset.name} role="option" aria-selected={preset.name === value} className="combo-option"
                title={preset.name} onClick={() => setOpen(false)}>
                {preset.colour && <span className="combo-option-swatch" style={{ background: preset.colour }} />}
                <span className="combo-text">{preset.name}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
