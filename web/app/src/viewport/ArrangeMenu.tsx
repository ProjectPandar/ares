import { useEffect, useState } from "react";
import { cmdDown, isShortcut } from "../shell/keys";
import { arrange, editArrangeSettings, resetArrangeSettings, showArrangeSettings } from "../state/arrange";
import { useApp } from "../state/store";

/** The spacing slider's range (`bbl_slider_float_style(..., 0.f, 100.0f, ...)`, GLCanvas3D.cpp:6708). */
const MAX_SPACING = 100;

/**
 * The main toolbar's "Arrange" item (GLCanvas3D.cpp:7567-7582), which toggles the arrange settings
 * popup (`_render_arrange_menu`, :6654-6778), and the shortcuts A (arrange all) and Shift+A (arrange
 * the current plate, Shortcuts.cpp:90-91).
 */
export function ArrangeMenu() {
  const state = useApp((app) => app.arrange);
  const busy = useApp((app) => app.arranging || app.slice.status === "slicing");
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!isShortcut(event) || cmdDown(event) || event.altKey || event.key.toLowerCase() !== "a") return;
      event.preventDefault();
      void arrange(event.shiftKey ? useApp.getState().currentPlate : undefined);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  if (!state) return null;
  const { settings, model } = state;
  const toggle = () => {
    if (!open) void showArrangeSettings();
    setOpen(!open);
  };
  const spacing = (value: number) => {
    if (!Number.isNaN(value)) void editArrangeSettings({ distance: value });
  };
  return (
    <div className="arrange-menu">
      <button className="view-button" aria-pressed={open} aria-haspopup="dialog" onClick={toggle}
        title={"Arrange all objects (A)\nArrange objects on selected plates (Shift+A)"}>Arrange</button>
      {open && (
        <div className="arrange-popup" role="dialog" aria-label="Arrange settings">
          <div className="arrange-spacing">
            <label htmlFor="arrange-spacing">Spacing</label>
            <input id="arrange-spacing" type="range" min={0} max={MAX_SPACING} step={0.01} value={Math.min(settings.distance, MAX_SPACING)}
              onChange={(event) => spacing(event.target.valueAsNumber)} />
            <input type="number" aria-label="Spacing value" min={0} step={0.05} value={settings.distance.toFixed(2)}
              onChange={(event) => spacing(event.target.valueAsNumber)} />
          </div>
          <p className="arrange-hint">0 means auto spacing.</p>
          <label>
            <input type="checkbox" checked={settings.enableRotation}
              onChange={(event) => void editArrangeSettings({ enableRotation: event.target.checked })} />
            Auto rotate for arrangement
          </label>
          <label>
            <input type="checkbox" checked={settings.allowMultiMaterialsOnSamePlate}
              onChange={(event) => void editArrangeSettings({ allowMultiMaterialsOnSamePlate: event.target.checked })} />
            Allow multiple materials on same plate
          </label>
          {model.showsCalibrationRegion && (
            <label>
              <input type="checkbox" checked={settings.avoidExtrusionCaliRegion}
                onChange={(event) => void editArrangeSettings({ avoidExtrusionCaliRegion: event.target.checked })} />
              Avoid extrusion calibration region
            </label>
          )}
          <label>
            <input type="checkbox" checked={settings.alignToYAxis} disabled={settings.enableRotation}
              onChange={(event) => void editArrangeSettings({ alignToYAxis: event.target.checked })} />
            Align to Y axis
          </label>
          <div className="arrange-actions">
            <button disabled={busy} onClick={() => void arrange()}>Arrange</button>
            <button onClick={resetArrangeSettings}>Reset</button>
          </div>
        </div>
      )}
    </div>
  );
}
