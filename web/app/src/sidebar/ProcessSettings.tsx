import type { ProjectSettings } from "@ares-slicer/wasm";
import { resetOverrides } from "../state/actions";
import { useApp } from "../state/store";
import { tabValues } from "../state/typedSettings";
import { SettingsPages } from "./SettingsPages";
import { PROCESS_PAGES } from "./processLayout";

export function ProcessSettings({ settings }: { settings: ProjectSettings | null }) {
  const overrides = useApp((state) => state.overrides);
  const typed = useApp((state) => state.typedSettings);
  const editedCount = Object.keys(overrides).length;
  return (
    <div className="process" data-testid="process-settings">
      {editedCount > 0 && (
        <div className="process-edited" data-testid="process-edited">
          <span>{editedCount} modified {editedCount === 1 ? "option" : "options"}</span>
          <button className="link-button" onClick={resetOverrides}>Reset all</button>
        </div>
      )}
      <SettingsPages pages={PROCESS_PAGES} empty="Settings appear after importing a 3MF project"
        context={settings && typed && { values: tabValues(settings, typed, "process"), overrides }} />
    </div>
  );
}
