import { useEffect, useId, useRef, useState } from "react";

export interface PresetEntry {
  name: string;
  /** Filament presets draw their `filament_colour` (PresetComboBox::get_bmp). */
  colour?: string;
}

/** A labelled group of the list (PlaterPresetComboBox::update `add_presets`). */
export interface PresetGroup {
  title: "Project presets" | "User presets" | "System presets";
  presets: PresetEntry[];
}

/** Trailing wizard item, e.g. "Select/Remove printers (system presets)". */
export interface ComboAction {
  label: string;
  run: () => void;
}

/**
 * PlaterPresetComboBox (PresetComboBoxes.cpp:1418-1468): project-embedded,
 * user and system groups in that order, then the wizard items.
 */
export function PresetCombo({ label, value, groups, placeholder, onSelect, actions = [] }: {
  label: string;
  value?: string;
  groups: PresetGroup[];
  placeholder: string;
  onSelect?: (name: string, group: PresetGroup["title"]) => void;
  actions?: ComboAction[];
}) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const listId = useId();
  const shown = groups.filter((group) => group.presets.length > 0);

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
        disabled={shown.length === 0 && actions.length === 0} onClick={() => setOpen(!open)}>
        <span className="combo-text">{value ?? placeholder}</span>
        <span className="combo-caret" />
      </button>
      {open && (
        <div className="combo-popup" role="listbox" id={listId} aria-label={label}>
          {shown.map((group) => (
            <div key={group.title} role="group" aria-label={group.title}>
              <div className="combo-group-title" role="presentation">{group.title}</div>
              {group.presets.map((preset) => (
                <div key={preset.name} role="option" aria-selected={preset.name === value} className="combo-option"
                  title={preset.name} onClick={() => {
                    setOpen(false);
                    onSelect?.(preset.name, group.title);
                  }}>
                  {preset.colour && <span className="combo-option-swatch" style={{ background: preset.colour }} />}
                  <span className="combo-text">{preset.name}</span>
                </div>
              ))}
            </div>
          ))}
          {actions.map((action) => (
            <div key={action.label} role="option" aria-selected={false} className="combo-option combo-action"
              onClick={() => {
                setOpen(false);
                action.run();
              }}>
              <span className="combo-text">{action.label}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
