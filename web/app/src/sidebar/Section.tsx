import type { ReactNode } from "react";
import { OrcaIcon } from "../shell/icons";
import type { PresetGroup } from "./PresetCombo";
import type { SettingsTab } from "./SettingsDialog";

export type OpenTab = (tab: SettingsTab) => void;

export /** Plater preset edit button (Plater.cpp `btn_edit_printer` / filament `edit_btn`). */
function EditButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button className="preset-edit" aria-label={label} title={label} onClick={onClick}>
      <OrcaIcon name="edit" />
    </button>
  );
}

export function Section({ icon, title, children, extra, action }: {
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

export const project = (name: string | undefined): PresetGroup => ({ title: "Project presets", presets: name ? [{ name }] : [] });

