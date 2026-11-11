import { lazy, Suspense, useState } from "react";
import { ObjectIcon } from "../shell/icons";
import { useApp } from "../state/store";
import { FilamentSection } from "./FilamentSection";
import { ObjectList } from "./ObjectList";
import { ObjectManipulation } from "./ObjectManipulation";
import { PrinterSection } from "./PrinterSection";
import { ProcessSection } from "./ProcessSection";
import { Section } from "./Section";
import type { SettingsTab } from "./SettingsDialog";

// Printer/filament tab layouts are only fetched when a preset edit button is used.
const SettingsDialog = lazy(() => import("./SettingsDialog").then((module) => ({ default: module.SettingsDialog })));

export function Sidebar() {
  const scene = useApp((state) => state.scene);
  const settings = scene?.settings ?? null;
  const [tab, setTab] = useState<SettingsTab | null>(null);
  // Phones show the sidebar as a drawer over the viewport; wider layouts always show it.
  const [open, setOpen] = useState(false);
  return (
    <>
    <button className="panel-toggle" aria-label="Settings panel" aria-controls="sidebar" aria-expanded={open}
      onClick={() => setOpen(!open)}>
      <span aria-hidden>{open ? "\u2715" : "\u2630"}</span>
    </button>
    <aside className="sidebar" id="sidebar" data-open={open} data-testid="sidebar">
      <PrinterSection settings={settings} openTab={setTab} />
      <FilamentSection settings={settings} openTab={setTab} />
      <Section icon={<ObjectIcon />} title="Objects" extra={scene ? <span data-testid="object-count">{scene.objects.length}</span> : undefined}>
        <ObjectList />
      </Section>
      <ObjectManipulation />
      <ProcessSection settings={settings} />
      {tab && settings && (
        <Suspense>
          <SettingsDialog tab={tab} settings={settings} onClose={() => setTab(null)} />
        </Suspense>
      )}
    </aside>
    </>
  );
}
