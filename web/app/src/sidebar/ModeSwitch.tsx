import type { CSSProperties, KeyboardEvent, MouseEvent } from "react";
import { setMode } from "../state/actions";
import { useApp, type ViewMode } from "../state/store";

const MODES: { mode: ViewMode; name: string }[] = [
  { mode: "simple", name: "Simple settings" },
  { mode: "advanced", name: "Advanced settings" },
  { mode: "expert", name: "Expert settings" },
];

const KEY_STEPS: Record<string, (selected: number) => number> = {
  ArrowLeft: (selected) => selected - 1,
  ArrowDown: (selected) => selected - 1,
  ArrowRight: (selected) => selected + 1,
  ArrowUp: (selected) => selected + 1,
  Home: () => 0,
  End: () => MODES.length - 1,
};

const select = (index: number) => setMode(MODES[Math.min(Math.max(index, 0), MODES.length - 1)].mode);

/**
 * `ModeSwitchButton` (Widgets/SwitchButton.cpp:250-446): one 48x18 track with a dot per mode. A
 * click selects the mode under its x third (`hit_test_selection`); the tooltip names the selection.
 */
export function ModeSwitch() {
  const current = useApp((state) => state.mode);
  const selected = MODES.findIndex(({ mode }) => mode === current);
  const { name } = MODES[selected];
  const onClick = (event: MouseEvent<HTMLElement>) => {
    const { left, width } = event.currentTarget.getBoundingClientRect();
    select(Math.floor(((event.clientX - left) * MODES.length) / width));
  };
  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    const step = KEY_STEPS[event.key];
    if (!step) return;
    event.preventDefault();
    select(step(selected));
  };
  return (
    <span className="mode-switch" role="slider" tabIndex={0} aria-label="Settings mode" aria-valuemin={0}
      aria-valuemax={MODES.length - 1} aria-valuenow={selected} aria-valuetext={name} title={name}
      style={{ "--mode": selected } as CSSProperties} onClick={onClick} onKeyDown={onKeyDown}>
      <span className="mode-track">
        {MODES.map(({ mode }, index) => (
          <span key={mode} className={`mode-dot${index <= selected ? " active" : ""}${index === selected ? " current" : ""}`} />
        ))}
      </span>
    </span>
  );
}
