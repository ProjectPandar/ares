import type { ProjectSettings } from "@ares-slicer/wasm";
import { lazy, Suspense, useState } from "react";
import { OrcaIcon } from "../shell/icons";
import { importedNames, selectFilament } from "../state/presets";
import { useApp } from "../state/store";
import { filamentGroups } from "./filamentGroups";
import { FilamentMapPanel } from "./FilamentMapPanel";
import { type PresetGroup, PresetCombo } from "./PresetCombo";
import { EditButton, type OpenTab, Section } from "./Section";
import { filamentNames, presetFilamentSlots, stringList } from "./options";

const FilamentDialog = lazy(() => import("./FilamentDialog").then((module) => ({ default: module.FilamentDialog })));

export function FilamentSection({ settings, openTab }: { settings: ProjectSettings | null; openTab: OpenTab }) {
  const imports = useApp((state) => state.presetImports);
  const selection = useApp((state) => state.presetSelection);
  const compatible = useApp((state) => state.compatible);
  const installed = useApp((state) => state.installedFilaments);
  const [dialog, setDialog] = useState(false);
  const names = filamentNames(settings);
  const colours = stringList(settings?.filament, "filament_colour");
  const types = stringList(settings?.filament, "filament_type");
  const slots = selection ? presetFilamentSlots(settings) : names.length;
  // Each preset once, drawn in the colour of its first slot.
  const presets = names
    .map((name, index) => ({ name, colour: colours[index] }))
    .filter(({ name }, index) => names.indexOf(name) === index);
  const user = importedNames(imports, "filament");
  const groups = (slot: number): PresetGroup[] => {
    if (!selection) return [{ title: "Project presets", presets }];
    if (compatible?.status !== "done") return [];
    return filamentGroups(compatible.presets.filaments, compatible.filamentInfo, user, installed, selection.filaments[slot]);
  };
  return (
    <Section icon={<OrcaIcon name="filament" />} title="Filament" extra={slots || undefined}>
      {slots === 0 && <div className="side-empty">No filament preset</div>}
      {Array.from({ length: slots }, (_, index) => names[index]).map((name, index) => (
        <div className="filament-row" key={index}>
          <span className="filament-swatch" data-testid={`filament-swatch-${index + 1}`}
            style={{ background: colours[index] ?? "#888" }}>{index + 1}</span>
          <PresetCombo label={`Filament ${index + 1} preset`} value={selection ? (selection.filaments[index] ?? "") : name} groups={groups(index)}
            placeholder="" searchable={selection !== null} onSelect={(filament) => selection && selectFilament(index, filament)}
            // PresetComboBoxes.cpp:1460-1461: the "Add/Remove filaments" item.
            actions={selection ? [{ label: "Add/Remove filaments", run: () => setDialog(true) }] : []} />
          <span className="filament-type">{types[index]}</span>
          {name !== undefined && (
            <EditButton label={`Edit filament ${index + 1} settings`} onClick={() => openTab({ kind: "filament", slot: index })} />
          )}
        </div>
      ))}
      <FilamentMapPanel settings={settings} />
      {dialog && selection && <Suspense><FilamentDialog printer={selection.printer} onClose={() => setDialog(false)} /></Suspense>}
    </Section>
  );
}
