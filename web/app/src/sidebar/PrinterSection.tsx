import type { ProjectSettings } from "@ares-slicer/wasm";
import { lazy, Suspense, useState } from "react";
import { OrcaIcon } from "../shell/icons";
import { importedNames, selectPrinter, selectPrinterModel } from "../state/presets";
import { type CompatibleState, useApp } from "../state/store";
import { ExtruderGroups } from "./ExtruderGroups";
import { PlateNozzle } from "./PlateNozzle";
import { PresetCombo } from "./PresetCombo";
import { EditButton, type OpenTab, project, Section } from "./Section";
import { firstString } from "./options";

const VendorDialog = lazy(() => import("./VendorDialog").then((module) => ({ default: module.VendorDialog })));

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

export function PrinterSection({ settings, openTab }: { settings: ProjectSettings | null; openTab: OpenTab }) {
  const imports = useApp((state) => state.presetImports);
  const selection = useApp((state) => state.presetSelection);
  const compatible = useApp((state) => state.compatible);
  const systemPrinters = useApp((state) => state.systemPrinters);
  const printerModels = useApp((state) => state.printerModels);
  const [wizard, setWizard] = useState(false);
  const printer = firstString(settings?.printer, "printer_settings_id") ?? firstString(settings?.project, "printer_settings_id");
  const user = [...importedNames(imports, "machine")].map((name) => ({ name }));
  // PlaterPresetComboBox::update: system printers as their models (PresetComboBoxes.cpp:1238-1251), a
  // selected one shown by its model with the preset name as the tooltip (:1241-1244, 1265, 1486).
  const system = printerModels.map((name) => ({ name }));
  const current = selection?.printer ?? printer;
  const selectedModel = systemPrinters.find(({ name }) => name === selection?.printer)?.printer_model;
  // PresetBundle.cpp:3755: the selected printer's printer_variant, a project printer's from its settings.
  const variant = systemPrinters.find(({ name }) => name === current)?.printer_variant ?? firstString(settings?.printer, "printer_variant") ?? "";
  return (
    <Section icon={<OrcaIcon name="printer" />} title="Printer">
      <div className="preset-row">
        <PresetCombo label="Printer preset" value={selectedModel ?? current} tooltip={current}
          groups={[project(printer), { title: "User presets", presets: user }, { title: "System presets", presets: system }]} placeholder="No printer preset"
          onSelect={(name, group) => void (group === "System presets" ? selectPrinterModel(name, { name: current ?? "", printer_variant: variant })
            : selectPrinter(group === "Project presets" ? null : name))}
          actions={[{ label: "Select/Remove printers (system presets)", run: () => setWizard(true) }]} />
        {settings && <EditButton label="Edit printer settings" onClick={() => openTab({ kind: "printer" })} />}
      </div>
      <CompatibleStatus compatible={compatible} />
      <PlateNozzle settings={settings} />
      <ExtruderGroups />
      {wizard && <Suspense><VendorDialog onClose={() => setWizard(false)} /></Suspense>}
    </Section>
  );
}

