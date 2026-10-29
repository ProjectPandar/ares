import { useEffect, useId, useRef, useState } from "react";

export interface PresetEntry {
  name: string;
  /** Filament presets draw their `filament_colour` (PresetComboBox::get_bmp). */
  colour?: string;
  /** Sub-menu of the entry; empty or unset shows it in the group itself (DropDown `group_key`). */
  subgroup?: string;
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
/**
 * A group's entries with each sub-menu at its first entry's position, holding every entry of
 * that sub-menu in list order (DropDown.cpp:357-384).
 */
type Row = PresetEntry | { subgroup: string; presets: PresetEntry[] };

function layout(presets: PresetEntry[]): Row[] {
  const rows: Row[] = [];
  const subgroups = new Map<string, PresetEntry[]>();
  for (const preset of presets) {
    if (!preset.subgroup) {
      rows.push(preset);
      continue;
    }
    const members = subgroups.get(preset.subgroup);
    if (members) members.push(preset);
    else {
      subgroups.set(preset.subgroup, [preset]);
      rows.push({ subgroup: preset.subgroup, presets: subgroups.get(preset.subgroup)! });
    }
  }
  return rows;
}

export function PresetCombo({ label, value, tooltip, groups, placeholder, onSelect, actions = [], searchable = false }: {
  label: string;
  value?: string;
  /** The selected preset's name when `value` shows its printer model (PresetComboBoxes.cpp:1265, 1486). */
  tooltip?: string;
  groups: PresetGroup[];
  placeholder: string;
  onSelect?: (name: string, group: PresetGroup["title"]) => void;
  actions?: ComboAction[];
  /** A search box over the entries: the filament dialog's lower-case substring match (guide/22/common.js:539-560). */
  searchable?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const root = useRef<HTMLDivElement>(null);
  const listId = useId();
  const query = search.trim().toLowerCase();
  const shown = groups
    .map((group) => ({ ...group, presets: group.presets.filter((preset) => preset.name.toLowerCase().includes(query)) }))
    .filter((group) => group.presets.length > 0);
  const option = (preset: PresetEntry, group: PresetGroup) => (
    <div key={preset.name} role="option" aria-selected={preset.name === value} className="combo-option"
      title={preset.name} onClick={() => {
        setOpen(false);
        onSelect?.(preset.name, group.title);
      }}>
      {preset.colour && <span className="combo-option-swatch" style={{ background: preset.colour }} />}
      <span className="combo-text">{preset.name}</span>
    </div>
  );

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
      <button type="button" className={value ? "combo" : "combo empty"} title={tooltip ?? value ?? placeholder} aria-label={label}
        aria-haspopup="listbox" aria-expanded={open} aria-controls={open ? listId : undefined}
        disabled={groups.every((group) => group.presets.length === 0) && actions.length === 0} onClick={() => {
          setSearch("");
          setOpen(!open);
        }}>
        <span className="combo-text">{value ?? placeholder}</span>
        <span className="combo-caret" />
      </button>
      {open && (
        <div className="combo-popup" role="listbox" id={listId} aria-label={label}>
          {searchable && (
            <input className="combo-search" type="search" aria-label={`Search ${label}`} placeholder="Search"
              autoFocus value={search} onChange={(event) => setSearch(event.target.value)} />
          )}
          {shown.map((group) => (
            <div key={group.title} role="group" aria-label={group.title}>
              <div className="combo-group-title" role="presentation">{group.title}</div>
              {layout(group.presets).map((row) => ("presets" in row
                ? (
                  <div key={`group:${row.subgroup}`} role="group" aria-label={row.subgroup} className="combo-subgroup">
                    <div className="combo-subgroup-title" role="presentation">{row.subgroup}</div>
                    {row.presets.map((preset) => option(preset, group))}
                  </div>
                )
                : option(row, group)))}
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
