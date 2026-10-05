import type { CSSProperties } from "react";
import { setMode } from "../state/actions";
import { useApp, type ViewMode } from "../state/store";

const MODES: { mode: ViewMode; name: string }[] = [
  { mode: "simple", name: "Simple settings" },
  { mode: "advanced", name: "Advanced settings" },
  { mode: "expert", name: "Expert settings" },
];

/** `ModeSwitchButton` (Widgets/SwitchButton.cpp:250-356): a 48x18 track with one dot per mode. */
export function ModeSwitch() {
  const current = useApp((state) => state.mode);
  const selected = MODES.findIndex(({ mode }) => mode === current);
  return (
    <span className="mode-switch" role="radiogroup" aria-label="Settings mode" style={{ "--mode": selected } as CSSProperties}>
      {MODES.map(({ mode, name }, index) => (
        <button key={mode} role="radio" aria-checked={index === selected} aria-label={name} title={name}
          className={`mode-dot${index <= selected ? " active" : ""}${index === selected ? " current" : ""}`}
          onClick={() => setMode(mode)} />
      ))}
    </span>
  );
}
