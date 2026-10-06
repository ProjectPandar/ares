import type { ProjectSettings } from "@ares-slicer/wasm";
import { lazy, type ReactNode, Suspense, useState } from "react";
import { ObjectIcon, OrcaIcon } from "../shell/icons";
import { importedNames, presetSliceBlocker, selectFilament, selectPrinter, selectProcess } from "../state/presets";
import { type CompatibleState, type PresetSelection, useApp } from "../state/store";
import { tabValues } from "../state/typedSettings";
import { ObjectList } from "./ObjectList";
import { type PresetGroup, PresetCombo } from "./PresetCombo";
import { filamentNames, firstString, presetFilamentSlots, stringList } from "./options";
import { ModeSwitch } from "./ModeSwitch";
import { ProcessSettings, usePresetValues } from "./ProcessSettings";
import type { SettingsTab } from "./SettingsDialog";

// Printer/filament tab layouts are only fetched when a preset edit button is used.
const SettingsDialog = lazy(() => import("./SettingsDialog").then((module) => ({ default: module.SettingsDialog })));
const VendorDialog = lazy(() => import("./VendorDialog").then((module) => ({ default: module.VendorDialog })));

type OpenTab = (tab: SettingsTab) => void;

/** Plater preset edit button (Plater.cpp `btn_edit_printer` / filament `edit_btn`). */
function EditButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button className="preset-edit" aria-label={label} title={label} onClick={onClick}>
      <OrcaIcon name="edit" />
    </button>
  );
}

function Section({ icon, title, children, extra, action }: {
  icon: ReactNode; title: string; children: ReactNode; extra?: ReactNode; action?: ReactNode;
}) {
  return (
    <section className="side-section">
      <header className="side-section-header">
        {icon}
        <span>{title}</span>
        {extra && <span className="side-section-extra">{extra}</span>}
        {action && <span className="side-section-action">{action}</span>}
      </header>
      <div className="side-section-body">{children}</div>
    </section>
  );
}

const project = (name: string | undefined): PresetGroup => ({ title: "Project presets", presets: name ? [{ name }] : [] });

/** Compatible names of one kind, imported ones under "User presets" and the rest under "System presets". */
function compatibleGroups(compatible: CompatibleState | null, names: "processes" | "filaments", user: Set<string>): PresetGroup[] {
  if (compatible?.status !== "done") return [];
  const all = compatible.presets[names];
  return [
    { title: "User presets", presets: all.filter((name) => user.has(name)).map((name) => ({ name })) },
    { title: "System presets", presets: all.filter((name) => !user.has(name)).map((name) => ({ name })) },
  ];
}

function CompatibleStatus({ compatible }: { compatible: CompatibleState | null }) {
  if (compatible?.status === "filtering") return <div className="side-empty" data-testid="preset-filtering">Filtering compatible presets…</div>;
  if (compatible?.status === "error") return <div className="preset-error" role="alert" data-testid="preset-error">{compatible.error}</div>;
  if (compatible?.status !== "done") return null;
  const { diagnostics, missing_parents: missingParents } = compatible.presets;
  return (
    <>
      {missingParents.length > 0 && (
        // Preset.cpp:1685-1691, PresetBundle.cpp:1521-1525: presets with an unloaded parent are skipped.
        <details className="preset-diagnostics" data-testid="preset-missing-parents">
          <summary>{missingParents.length} preset(s) skipped: parent preset not loaded</summary>
          <ul>{missingParents.map(({ kind, name, parent }) => <li key={`${kind}:${name}`}>{name} ({kind}): parent preset &quot;{parent}&quot; is not loaded</li>)}</ul>
        </details>
      )}
      {diagnostics.length > 0 && (
        <details className="preset-diagnostics" data-testid="preset-diagnostics">
          <summary>{diagnostics.length} compatibility condition(s) could not be evaluated</summary>
          <ul>{diagnostics.map((item) => <li key={`${item.kind}:${item.name}`}>{item.name}: {item.message}</li>)}</ul>
        </details>
      )}
    </>
  );
}

function PrinterSection({ settings, openTab }: { settings: ProjectSettings | null; openTab: OpenTab }) {
  const imports = useApp((state) => state.presetImports);
  const selection = useApp((state) => state.presetSelection);
  const compatible = useApp((state) => state.compatible);
  const systemPrinters = useApp((state) => state.systemPrinters);
  const [wizard, setWizard] = useState(false);
  const nozzle = firstString(settings?.project, "nozzle_diameter");
  const bed = firstString(settings?.project, "curr_bed_type") ?? firstString(settings?.printer, "default_bed_type");
  const printer = firstString(settings?.printer, "printer_settings_id") ?? firstString(settings?.project, "printer_settings_id");
  const user = [...importedNames(imports, "machine")].map((name) => ({ name }));
  // TabPresetComboBox::update (PresetComboBoxes.cpp:1773, 1807): user presets, then system preset names.
  const system = systemPrinters.map(({ name }) => ({ name }));
  return (
    <Section icon={<OrcaIcon name="printer" />} title="Printer">
      <div className="preset-row">
        <PresetCombo label="Printer preset" value={selection?.printer ?? printer}
          groups={[project(printer), { title: "User presets", presets: user }, { title: "System presets", presets: system }]} placeholder="No printer preset"
          onSelect={(name, group) => void selectPrinter(group === "Project presets" ? null : name)}
          actions={[{ label: "Select/Remove printers (system presets)", run: () => setWizard(true) }]} />
        {settings && <EditButton label="Edit printer settings" onClick={() => openTab({ kind: "printer" })} />}
      </div>
      <CompatibleStatus compatible={compatible} />
      <div className="kv-row"><span>Plate type</span><span className="kv-value">{bed ?? "—"}</span></div>
      <div className="kv-row"><span>Nozzle</span><span className="kv-value">{nozzle ? `${nozzle} mm` : "—"}</span></div>
      {wizard && <Suspense><VendorDialog onClose={() => setWizard(false)} /></Suspense>}
    </Section>
  );
}

function FilamentSection({ settings, openTab }: { settings: ProjectSettings | null; openTab: OpenTab }) {
  const imports = useApp((state) => state.presetImports);
  const selection = useApp((state) => state.presetSelection);
  const compatible = useApp((state) => state.compatible);
  const names = filamentNames(settings);
  const colours = stringList(settings?.filament, "filament_colour");
  const types = stringList(settings?.filament, "filament_type");
  const slots = selection ? presetFilamentSlots(settings) : names.length;
  // Each preset once, drawn in the colour of its first slot.
  const presets = names
    .map((name, index) => ({ name, colour: colours[index] }))
    .filter(({ name }, index) => names.indexOf(name) === index);
  const groups = selection
    ? compatibleGroups(compatible, "filaments", importedNames(imports, "filament"))
    : [{ title: "Project presets" as const, presets }];
  return (
    <Section icon={<OrcaIcon name="filament" />} title="Filament" extra={slots || undefined}>
      {slots === 0 && <div className="side-empty">No filament preset</div>}
      {Array.from({ length: slots }, (_, index) => names[index]).map((name, index) => (
        <div className="filament-row" key={index}>
          <span className="filament-swatch" data-testid={`filament-swatch-${index + 1}`}
            style={{ background: colours[index] ?? "#888" }}>{index + 1}</span>
          <PresetCombo label={`Filament ${index + 1} preset`} value={selection?.filaments[index] ?? name} groups={groups}
            placeholder="" onSelect={(filament) => selection && selectFilament(index, filament)} />
          <span className="filament-type">{types[index]}</span>
          {name !== undefined && (
            <EditButton label={`Edit filament ${index + 1} settings`} onClick={() => openTab({ kind: "filament", slot: index })} />
          )}
        </div>
      ))}
    </Section>
  );
}

/** Process tab of the selected presets, editable like the project's (Tab::update after a preset select). */
function PresetProcessSettings({ selection }: { selection: PresetSelection }) {
  const { values, error } = usePresetValues(selection);
  return <ProcessSettings values={values} empty={error ?? "Settings appear once the presets are selected"} />;
}

function ProcessSection({ settings }: { settings: ProjectSettings | null }) {
  const scene = useApp((state) => state.scene);
  const imports = useApp((state) => state.presetImports);
  const selection = useApp((state) => state.presetSelection);
  const compatible = useApp((state) => state.compatible);
  const typed = useApp((state) => state.typedSettings);
  const process = firstString(settings?.process, "print_settings_id") ?? firstString(settings?.project, "print_settings_id");
  return (
    // ParamsPanel top panel: the process tab's ModeSwitchButton (ParamsPanel.cpp:281).
    <Section icon={<OrcaIcon name="process" />} title="Process" action={<ModeSwitch />}>
      {selection ? (
        <>
          <PresetCombo label="Process preset" value={selection.process} placeholder="Select a process preset"
            groups={compatibleGroups(compatible, "processes", importedNames(imports, "process"))}
            onSelect={(name) => void selectProcess(name)} />
          {scene && presetSliceBlocker(selection, scene) && (
            <div className="side-empty" data-testid="preset-slice-blocker">{presetSliceBlocker(selection, scene)}.</div>
          )}
          <PresetProcessSettings selection={selection} />
        </>
      ) : (
        <>
          <PresetCombo label="Process preset" value={process} groups={[project(process)]} placeholder="No process preset" />
          <ProcessSettings values={settings && typed && tabValues(settings, typed, "process")}
            empty="Settings appear after importing a 3MF project" />
        </>
      )}
    </Section>
  );
}

export function Sidebar() {
  const scene = useApp((state) => state.scene);
  const settings = scene?.settings ?? null;
  const [tab, setTab] = useState<SettingsTab | null>(null);
  return (
    <aside className="sidebar" data-testid="sidebar">
      <PrinterSection settings={settings} openTab={setTab} />
      <FilamentSection settings={settings} openTab={setTab} />
      <Section icon={<ObjectIcon />} title="Objects" extra={scene ? <span data-testid="object-count">{scene.objects.length}</span> : undefined}>
        <ObjectList />
      </Section>
      <ProcessSection settings={settings} />
      {tab && settings && (
        <Suspense>
          <SettingsDialog tab={tab} settings={settings} onClose={() => setTab(null)} />
        </Suspense>
      )}
    </aside>
  );
}
