import type { ProjectSettings } from "@ares-slicer/wasm";
import { OrcaIcon } from "../shell/icons";
import { importedNames, presetSliceBlocker, selectProcess } from "../state/presets";
import { type CompatibleState, type PresetSelection, useApp } from "../state/store";
import { tabValues } from "../state/typedSettings";
import { ModeSwitch } from "./ModeSwitch";
import { type PresetGroup, PresetCombo } from "./PresetCombo";
import { ProcessSettings, usePresetValues } from "./ProcessSettings";
import { project, Section } from "./Section";
import { firstString } from "./options";

/** Compatible names of one kind, imported ones under "User presets" and the rest under "System presets". */
function compatibleGroups(compatible: CompatibleState | null, names: "processes" | "filaments", user: Set<string>): PresetGroup[] {
  if (compatible?.status !== "done") return [];
  const all = compatible.presets[names];
  return [
    { title: "User presets", presets: all.filter((name) => user.has(name)).map((name) => ({ name })) },
    { title: "System presets", presets: all.filter((name) => !user.has(name)).map((name) => ({ name })) },
  ];
}

/** Process tab of the selected presets, editable like the project's (Tab::update after a preset select). */
function PresetProcessSettings({ selection }: { selection: PresetSelection }) {
  const { values, error } = usePresetValues(selection);
  return <ProcessSettings values={values} empty={error ?? "Settings appear once the presets are selected"} />;
}

export function ProcessSection({ settings }: { settings: ProjectSettings | null }) {
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

