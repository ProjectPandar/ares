import type { ProjectSettings } from "@ares-slicer/wasm";
import type { ReactNode } from "react";
import { FilamentIcon, ObjectIcon, PrinterIcon, ProcessIcon } from "../shell/icons";
import { useApp } from "../state/store";
import { ObjectList } from "./ObjectList";
import { firstString, stringList } from "./options";
import { ProcessSettings } from "./ProcessSettings";

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

function PresetCombo({ value, placeholder }: { value?: string; placeholder: string }) {
  return (
    <div className={value ? "combo" : "combo empty"} title={value ?? placeholder}>
      <span className="combo-text">{value ?? placeholder}</span>
      <span className="combo-caret" />
    </div>
  );
}

function PrinterSection({ settings }: { settings: ProjectSettings | null }) {
  const nozzle = firstString(settings?.project, "nozzle_diameter");
  const bed = firstString(settings?.project, "curr_bed_type") ?? firstString(settings?.printer, "default_bed_type");
  return (
    <Section icon={<PrinterIcon />} title="Printer">
      <PresetCombo value={firstString(settings?.printer, "printer_settings_id") ?? firstString(settings?.project, "printer_settings_id")}
        placeholder="No printer preset" />
      <div className="kv-row"><span>Plate type</span><span className="kv-value">{bed ?? "—"}</span></div>
      <div className="kv-row"><span>Nozzle</span><span className="kv-value">{nozzle ? `${nozzle} mm` : "—"}</span></div>
    </Section>
  );
}

function FilamentSection({ settings }: { settings: ProjectSettings | null }) {
  const names = stringList(settings?.filament, "filament_settings_id").length
    ? stringList(settings?.filament, "filament_settings_id")
    : stringList(settings?.project, "filament_settings_id");
  const colours = stringList(settings?.filament, "filament_colour");
  const types = stringList(settings?.filament, "filament_type");
  return (
    <Section icon={<FilamentIcon />} title="Filament" extra={names.length ? names.length : undefined}>
      {names.length === 0 && <div className="side-empty">No filament preset</div>}
      {names.map((name, index) => (
        <div className="filament-row" key={index}>
          <span className="filament-swatch" style={{ background: colours[index] ?? "#888" }}>{index + 1}</span>
          <PresetCombo value={name} placeholder="" />
          <span className="filament-type">{types[index]}</span>
        </div>
      ))}
    </Section>
  );
}

export function Sidebar() {
  const scene = useApp((state) => state.scene);
  const settings = scene?.settings ?? null;
  return (
    <aside className="sidebar" data-testid="sidebar">
      <PrinterSection settings={settings} />
      <FilamentSection settings={settings} />
      <Section icon={<ObjectIcon />} title="Objects" extra={scene ? scene.objects.length : undefined}>
        <ObjectList />
      </Section>
      <Section icon={<ProcessIcon />} title="Process">
        <PresetCombo value={firstString(settings?.process, "print_settings_id") ?? firstString(settings?.project, "print_settings_id")}
          placeholder="No process preset" />
        <ProcessSettings settings={settings} />
      </Section>
    </aside>
  );
}
