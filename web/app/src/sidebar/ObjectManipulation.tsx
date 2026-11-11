import { useEffect, useState } from "react";
import { ObjectIcon } from "../shell/icons";
import type { ScaleAndSize } from "@ares-slicer/wasm";
import { rotateInstance, setInstancePosition, setInstanceRotation, setInstanceScale, setInstanceSize } from "../state/plates";
import { useApp } from "../state/store";
import { takeSnapshot } from "../state/undoRedo";
import { Section } from "./Section";

const AXES = ["X", "Y", "Z"] as const;
type Axis = 0 | 1 | 2;

/** `reset_scale_value` (GizmoObjectManipulation.cpp:609-616): each axis's Scale field set to 100. */
async function resetScale(target: { object: number; instance: number }, uniform: boolean) {
  for (const axis of [0, 1, 2] as const) await setInstanceScale(target, { axis, value: 100, uniform });
}

/** A coordinate field committed on Enter or blur; external changes replace the text. */
function CoordinateField({ name, label, value, unit, disabled, onCommit }: {
  name: string; label: string; value: number; unit: string; disabled: boolean; onCommit: (value: number) => void;
}) {
  // GizmoObjectManipulation shows two decimals (`%.2f`).
  const shown = value.toFixed(2);
  const [draft, setDraft] = useState<{ shown: string; text: string } | null>(null);
  const text = draft?.shown === shown ? draft.text : shown;
  const commit = () => {
    setDraft(null);
    const parsed = Number(text);
    if (text.trim() !== "" && Number.isFinite(parsed) && text !== shown) onCommit(parsed);
  };
  return (
    <label className="manipulation-field">
      <span>{label}</span>
      <input className="option-field" aria-label={`${name} ${label}`} value={text} disabled={disabled} inputMode="decimal"
        onChange={(event) => setDraft({ shown, text: event.target.value })}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === "Enter") commit();
        }} />
      <span className="manipulation-unit">{unit}</span>
    </label>
  );
}

function Fields({ name, values, unit, disabled, onCommit }: {
  name: string; values: readonly number[]; unit: string; disabled: boolean; onCommit: (axis: Axis, value: number) => void;
}) {
  return (
    <div className="manipulation" role="group" aria-label={name}>
      <span className="manipulation-title">{name}</span>
      {AXES.map((label, axis) => (
        <CoordinateField key={label} name={name} label={label} value={values[axis]} unit={unit} disabled={disabled}
          onCommit={(value) => onCommit(axis as Axis, value)} />
      ))}
    </div>
  );
}

/**
 * GizmoObjectManipulation for the selected object's first instance in world coordinates: its
 * position (`get_instance_offset`, `change_position_value`), a relative rotation applied as
 * entered (`change_rotation_value`, shown back at zero), its absolute rotation
 * (`change_absolute_rotation_value`), and its scale and size (`change_scale_value`,
 * `change_size_value`) with the "Uniform scale" toggle (on by default, `m_uniform_scale`).
 */
export function ObjectManipulation() {
  const scene = useApp((state) => state.scene);
  const plater = useApp((state) => state.plater);
  const selected = useApp((state) => state.selectedObject);
  const slicing = useApp((state) => state.slice.status === "slicing");
  const object = scene && plater ? scene.objects.findIndex(({ id }) => id === selected) : -1;
  const matrix = object < 0 ? null : scene!.objects[object].instances[0].matrix;
  const [fields, setFields] = useState<{ matrix: readonly number[]; degrees: number[] } & ScaleAndSize | null>(null);
  const [uniform, setUniform] = useState(true);
  useEffect(() => {
    if (!plater || !matrix) return;
    let stale = false;
    void Promise.all([plater.absoluteRotation(object, 0), plater.scaleAndSize(object, 0)]).then(
      ([degrees, scaleAndSize]) => !stale && setFields({ matrix, degrees, ...scaleAndSize }),
    );
    return () => {
      stale = true;
    };
  }, [plater, object, matrix]);
  if (!matrix) return null;
  const target = { object, instance: 0 };
  const radians = (axis: Axis, degrees: number) => [0, 1, 2].map((index) => (index === axis ? (degrees * Math.PI) / 180 : 0)) as [number, number, number];
  return (
    <Section icon={<ObjectIcon />} title="Object operations">
      <Fields name="Position" values={[matrix[12], matrix[13], matrix[14]]} unit="mm" disabled={slicing}
        onCommit={(axis, value) => void setInstancePosition(target, axis, value)} />
      <Fields name="Rotate (relative)" values={[0, 0, 0]} unit="°" disabled={slicing}
        onCommit={(axis, degrees) => {
          takeSnapshot();
          void rotateInstance(target, radians(axis, degrees));
        }} />
      {fields?.matrix === matrix && (
        <>
          <Fields name="Rotate (absolute)" values={fields.degrees} unit="°" disabled={slicing}
            onCommit={(axis, degrees) => void setInstanceRotation(target, axis, degrees)} />
          <Fields name="Scale" values={fields.scale} unit="%" disabled={slicing}
            onCommit={(axis, value) => void setInstanceScale(target, { axis, value, uniform })} />
          {/* m_show_clear_scale (GizmoObjectManipulation.cpp:299); reset_scale_value sets each axis to 100. */}
          {Math.hypot(...fields.scale.map((value) => value / 100 - 1)) > 0.001 && (
            <button type="button" className="side-button manipulation-reset" disabled={slicing}
              onClick={() => void resetScale(target, uniform)}>Reset scale</button>
          )}
          <Fields name="Size" values={fields.size} unit="mm" disabled={slicing}
            onCommit={(axis, value) => void setInstanceSize(target, { axis, value, uniform })} />
          <label className="manipulation-toggle">
            <input type="checkbox" checked={uniform} disabled={slicing} onChange={(event) => setUniform(event.target.checked)} />
            Uniform scale
          </label>
        </>
      )}
    </Section>
  );
}
