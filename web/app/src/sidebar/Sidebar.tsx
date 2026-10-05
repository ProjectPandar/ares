import type { ProjectSettings } from "@ares-slicer/wasm";
import { lazy, type ReactNode, Suspense, useState } from "react";
import { ObjectIcon, OrcaIcon } from "../shell/icons";
import { useApp } from "../state/store";
import { ObjectList } from "./ObjectList";
import { PresetCombo } from "./PresetCombo";
import { firstString, stringList } from "./options";
import { ProcessSettings } from "./ProcessSettings";
import type { SettingsTab } from "./SettingsDialog";

// Printer/filament tab layouts are only fetched when a preset edit button is used.
const SettingsDialog = lazy(() => import("./SettingsDialog").then((module) => ({ default: module.SettingsDialog })));

type OpenTab = (tab: SettingsTab) => void;

/** Plater preset edit button (Plater.cpp `btn_edit_printer` / filament `edit_btn`). */
function EditButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button className="preset-edit" aria-label={label} title={label} onClick={onClick}>
      <OrcaIcon name="edit" />
    </button>
  );
}

function Section({ icon, title, children, extra }: {
  icon: ReactNode; title: string; children: ReactNode; extra?: ReactNode;
}) {
  return (
    <section className="side-section">
      <header className="side-section-header">
        {icon}
        <span>{title}</span>
        {extra && <span className="side-section-extra">{extra}</span>}
      </header>
      <div className="side-section-body">{children}</div>
    </section>
  );
}

function PrinterSection({ settings, openTab }: { settings: ProjectSettings | null; openTab: OpenTab }) {
  const nozzle = firstString(settings?.project, "nozzle_diameter");
  const bed = firstString(settings?.project, "curr_bed_type") ?? firstString(settings?.printer, "default_bed_type");
  const printer = firstString(settings?.printer, "printer_settings_id") ?? firstString(settings?.project, "printer_settings_id");
  return (
    <Section icon={<OrcaIcon name="printer" />} title="Printer">
      <div className="preset-row">
        <PresetCombo label="Printer preset" value={printer} presets={printer ? [{ name: printer }] : []}
          placeholder="No printer preset" />
        {settings && <EditButton label="Edit printer settings" onClick={() => openTab({ kind: "printer" })} />}
      </div>
      <div className="kv-row"><span>Plate type</span><span className="kv-value">{bed ?? "—"}</span></div>
      <div className="kv-row"><span>Nozzle</span><span className="kv-value">{nozzle ? `${nozzle} mm` : "—"}</span></div>
    </Section>
  );
}

function FilamentSection({ settings, openTab }: { settings: ProjectSettings | null; openTab: OpenTab }) {
  const names = stringList(settings?.filament, "filament_settings_id").length
    ? stringList(settings?.filament, "filament_settings_id")
    : stringList(settings?.project, "filament_settings_id");
  const colours = stringList(settings?.filament, "filament_colour");
  const types = stringList(settings?.filament, "filament_type");
  // Each preset once, drawn in the colour of its first slot.
  const presets = names
    .map((name, index) => ({ name, colour: colours[index] }))
    .filter(({ name }, index) => names.indexOf(name) === index);
  return (
    <Section icon={<OrcaIcon name="filament" />} title="Filament" extra={names.length ? names.length : undefined}>
      {names.length === 0 && <div className="side-empty">No filament preset</div>}
      {names.map((name, index) => (
        <div className="filament-row" key={index}>
          <span className="filament-swatch" data-testid={`filament-swatch-${index + 1}`}
            style={{ background: colours[index] ?? "#888" }}>{index + 1}</span>
          <PresetCombo label={`Filament ${index + 1} preset`} value={name} presets={presets} placeholder="" />
          <span className="filament-type">{types[index]}</span>
          <EditButton label={`Edit filament ${index + 1} settings`} onClick={() => openTab({ kind: "filament", slot: index })} />
        </div>
      ))}
    </Section>
  );
}

export function Sidebar() {
  const scene = useApp((state) => state.scene);
  const settings = scene?.settings ?? null;
  const process = firstString(settings?.process, "print_settings_id") ?? firstString(settings?.project, "print_settings_id");
  const [tab, setTab] = useState<SettingsTab | null>(null);
  return (
    <aside className="sidebar" data-testid="sidebar">
      <PrinterSection settings={settings} openTab={setTab} />
      <FilamentSection settings={settings} openTab={setTab} />
      <Section icon={<ObjectIcon />} title="Objects" extra={scene ? <span data-testid="object-count">{scene.objects.length}</span> : undefined}>
        <ObjectList />
      </Section>
      <Section icon={<OrcaIcon name="process" />} title="Process">
        <PresetCombo label="Process preset" value={process} presets={process ? [{ name: process }] : []}
          placeholder="No process preset" />
        <ProcessSettings settings={settings} />
      </Section>
      {tab && settings && (
        <Suspense>
          <SettingsDialog tab={tab} settings={settings} onClose={() => setTab(null)} />
        </Suspense>
      )}
    </aside>
  );
}
