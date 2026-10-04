import type { ProjectSettings } from "@ares-slicer/wasm";
import { useState } from "react";
import { resetOverrides } from "../state/actions";
import { useApp } from "../state/store";
import { OrcaIcon } from "../shell/icons";
import { OptionField } from "./OptionField";
import { OPTION_LABELS, PROCESS_PAGES } from "./processLayout";

export function ProcessSettings({ settings }: { settings: ProjectSettings | null }) {
  const [page, setPage] = useState(PROCESS_PAGES[0].title);
  const overrides = useApp((state) => state.overrides);
  const values = { ...settings?.project, ...settings?.process };
  const current = PROCESS_PAGES.find((candidate) => candidate.title === page) ?? PROCESS_PAGES[0];
  const editedCount = Object.keys(overrides).length;
  return (
    <div className="process">
      {editedCount > 0 && (
        <div className="process-edited" data-testid="process-edited">
          <span>{editedCount} modified {editedCount === 1 ? "option" : "options"}</span>
          <button className="link-button" onClick={resetOverrides}>Reset all</button>
        </div>
      )}
      <div className="process-tabs" role="tablist">
        {PROCESS_PAGES.map((candidate) => (
          <button key={candidate.title} role="tab" aria-selected={candidate.title === page}
            className={candidate.title === page ? "process-tab active" : "process-tab"}
            onClick={() => setPage(candidate.title)}>
            {candidate.title}
          </button>
        ))}
      </div>
      {!settings && <div className="side-empty">Settings appear after importing a 3MF project</div>}
      {settings && current.groups.map((group) => {
        const keys = group.keys.filter((key) => key in values);
        if (keys.length === 0) return null;
        return (
          <div className="optgroup" key={group.title}>
            <div className="optgroup-title">{group.icon && <OrcaIcon name={group.icon} size={18} />}{group.title}</div>
            {keys.map((key) => (
              <OptionField key={key} optionKey={key} label={OPTION_LABELS[key]} original={values[key]} edited={overrides[key]} />
            ))}
          </div>
        );
      })}
    </div>
  );
}
