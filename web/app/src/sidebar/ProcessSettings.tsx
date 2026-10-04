import type { ProjectSettings } from "@ares-slicer/wasm";
import { useState } from "react";
import { formatOption, isBoolean } from "./options";
import { OPTION_LABELS, PROCESS_PAGES } from "./processLayout";

const READ_ONLY_HINT = "Read-only: editing needs an ares-core option override API";

export function ProcessSettings({ settings }: { settings: ProjectSettings | null }) {
  const [page, setPage] = useState(PROCESS_PAGES[0].title);
  const values = { ...settings?.project, ...settings?.process };
  const current = PROCESS_PAGES.find((candidate) => candidate.title === page) ?? PROCESS_PAGES[0];
  return (
    <div className="process">
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
            <div className="optgroup-title">{group.title}</div>
            {keys.map((key) => {
              const label = OPTION_LABELS[key];
              const value = values[key];
              return (
                <div className="option-row" key={key} title={`${key}\n${READ_ONLY_HINT}`}>
                  <span className="option-label">{label?.label ?? key}</span>
                  {isBoolean(value) ? (
                    <input type="checkbox" checked={value} readOnly className="option-check" />
                  ) : (
                    <span className="option-value">
                      <span className="option-field">{formatOption(value)}</span>
                      {label?.sidetext && <span className="option-unit">{label.sidetext}</span>}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}
