import type { OptionMap } from "@ares-slicer/wasm";
import { useEffect, useState } from "react";
import { resetOverrides } from "../state/actions";
import { presetValues } from "../state/presetSlice";
import { presetSliceBlocker } from "../state/presets";
import { type ModelFile, type PresetSelection, useApp } from "../state/store";

type Fetched = { file: ModelFile; selection: PresetSelection } & ({ values: OptionMap } | { error: string });

/**
 * The process tab's values with presets selected: the config the preset slice composes,
 * refetched when the selection, the model or the catalog changes.
 */
export function usePresetValues(selection: PresetSelection): { values: OptionMap | null; error?: string } {
  const file = useApp((state) => state.file);
  const scene = useApp((state) => state.scene);
  const vendors = useApp((state) => state.vendors);
  const presetImports = useApp((state) => state.presetImports);
  const [fetched, setFetched] = useState<Fetched | null>(null);
  const blocked = !scene || presetSliceBlocker(selection, scene) !== null;
  useEffect(() => {
    if (!file || !scene || blocked) return;
    let stale = false;
    presetValues(file, scene, selection, { vendors, presetImports }).then(
      (values) => !stale && setFetched({ file, selection, values }),
      (error: Error) => !stale && setFetched({ file, selection, error: error.message }),
    );
    return () => {
      stale = true;
    };
  }, [file, scene, blocked, selection, vendors, presetImports]);
  if (blocked || fetched?.file !== file || fetched.selection !== selection) return { values: null };
  return "values" in fetched ? { values: fetched.values } : { values: null, error: fetched.error };
}
import { SettingsPages } from "./SettingsPages";
import { PROCESS_PAGES } from "./processLayout";

/** The process tab's values (`values`) with the sidebar edits on top. */
export function ProcessSettings({ values, empty }: { values: OptionMap | null; empty: string }) {
  const overrides = useApp((state) => state.overrides);
  const editedCount = Object.keys(overrides).length;
  return (
    <div className="process" data-testid="process-settings">
      {editedCount > 0 && (
        <div className="process-edited" data-testid="process-edited">
          <span>{editedCount} modified {editedCount === 1 ? "option" : "options"}</span>
          <button className="link-button" onClick={resetOverrides}>Reset all</button>
        </div>
      )}
      <SettingsPages pages={PROCESS_PAGES} empty={empty} context={values && { values, overrides }} />
    </div>
  );
}
