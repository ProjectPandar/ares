import { useEffect } from "react";
import { OrcaIcon } from "../shell/icons";
import { cmdDown, isShortcut } from "../shell/keys";
import { arrange, editArrangeSettings, resetArrangeSettings, showArrangeSettings } from "../state/arrange";
import { useApp } from "../state/store";

/** The spacing slider's range (`bbl_slider_float_style(..., 0.f, 100.0f, ...)`, GLCanvas3D.cpp:6708). */
const MAX_SPACING = 100;

/**
 * The main toolbar's "Arrange" item (GLCanvas3D.cpp:7567-7582), enabled by `can_arrange` (objects,
 * no arrange running), which toggles the arrange settings popup, and the shortcuts A (arrange all)
 * and Shift+A (arrange the current plate, Shortcuts.cpp:90-91).
 */
export function ArrangeButton({
  open,
  onToggle,
  icon,
}: {
  open: boolean;
  onToggle: (open: boolean) => void;
  icon: string;
}) {
  const ready = useApp((app) => !!app.arrange && !!app.scene?.objects.length && !app.arranging);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!isShortcut(event) || cmdDown(event) || event.altKey || event.key.toLowerCase() !== "a") return;
      event.preventDefault();
      void arrange(event.shiftKey ? useApp.getState().currentPlate : undefined);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <button
      type="button"
      className="toolbar-item"
      aria-label="Arrange"
      aria-pressed={open}
      aria-haspopup="dialog"
      disabled={!ready}
      title={"Arrange all objects [A]\nArrange objects on selected plates [Shift+A]"}
      onClick={() => {
        if (!open) void showArrangeSettings();
        onToggle(!open);
      }}
    >
      <OrcaIcon name={icon} size={24} />
    </button>
  );
}

/** The arrange settings popup (`_render_arrange_menu`, GLCanvas3D.cpp:6654-6778). */
export function ArrangePopup({ onClose }: { onClose: () => void }) {
  const state = useApp((app) => app.arrange);
  const busy = useApp((app) => app.arranging || app.slice.status === "slicing");
  if (!state) return null;
  const { settings, model } = state;
  const spacing = (value: number) => {
    if (!Number.isNaN(value)) void editArrangeSettings({ distance: value });
  };
  const option = (name: string, checked: boolean, change: (value: boolean) => void, disabled = false) => (
    <label className="check">
      <input type="checkbox" checked={checked} disabled={disabled} onChange={(event) => change(event.target.checked)} />
      <span>{name}</span>
    </label>
  );
  return (
    <div className="popup" role="dialog" aria-labelledby="arrange-settings-title">
      <header className="dialog-header">
        <div className="dialog-heading">
          <h1 id="arrange-settings-title">Arrange settings</h1>
        </div>
        <button className="button close" type="button" aria-label="Close" onClick={onClose}>
          {"×"}
        </button>
      </header>
      <div className="dialog-body">
        <div className="stack">
          <div className="spacing-row">
            <label className="field">
              <span>Spacing</span>
              <input
                id="arrange-spacing"
                type="range"
                min={0}
                max={MAX_SPACING}
                step={0.01}
                value={Math.min(settings.distance, MAX_SPACING)}
                onChange={(event) => spacing(event.target.valueAsNumber)}
              />
            </label>
            <label className="field">
              <span>mm</span>
              <input
                type="number"
                aria-label="Spacing value"
                min={0}
                step={0.05}
                value={settings.distance.toFixed(2)}
                onChange={(event) => spacing(event.target.valueAsNumber)}
              />
            </label>
          </div>
          <p className="muted">0 means auto spacing.</p>
          {option(
            "Auto rotate for arrangement",
            settings.enableRotation,
            (enableRotation) => void editArrangeSettings({ enableRotation }),
          )}
          {option(
            "Allow multiple materials on same plate",
            settings.allowMultiMaterialsOnSamePlate,
            (allowMultiMaterialsOnSamePlate) => void editArrangeSettings({ allowMultiMaterialsOnSamePlate }),
          )}
          {model.showsCalibrationRegion &&
            option(
              "Avoid extrusion calibration region",
              settings.avoidExtrusionCaliRegion,
              (avoidExtrusionCaliRegion) => void editArrangeSettings({ avoidExtrusionCaliRegion }),
            )}
          {option(
            "Align to Y axis",
            settings.alignToYAxis,
            (alignToYAxis) => void editArrangeSettings({ alignToYAxis }),
            settings.enableRotation,
          )}
        </div>
      </div>
      <footer className="dialog-footer">
        <button className="button" onClick={resetArrangeSettings}>
          Reset
        </button>
        <button className="button primary" disabled={busy} onClick={() => void arrange()}>
          Arrange
        </button>
      </footer>
    </div>
  );
}
