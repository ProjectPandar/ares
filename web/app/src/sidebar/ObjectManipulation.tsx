import { useEffect, useState } from "react";
import { ObjectIcon } from "../shell/icons";
import { rotateInstance, setInstancePosition, setInstanceRotation } from "../state/plates";
import { useApp } from "../state/store";
import { Section } from "./Section";

const AXES = ["X", "Y", "Z"] as const;
type Axis = 0 | 1 | 2;

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
 * entered (`change_rotation_value`, shown back at zero), and its absolute rotation
 * (`change_absolute_rotation_value`).
 */
export function ObjectManipulation() {
  const scene = useApp((state) => state.scene);
  const plater = useApp((state) => state.plater);
  const selected = useApp((state) => state.selectedObject);
  const slicing = useApp((state) => state.slice.status === "slicing");
  const object = scene && plater ? scene.objects.findIndex(({ id }) => id === selected) : -1;
  const matrix = object < 0 ? null : scene!.objects[object].instances[0].matrix;
  const [rotation, setRotation] = useState<{ matrix: readonly number[]; degrees: number[] } | null>(null);
  useEffect(() => {
    if (!plater || !matrix) return;
    let stale = false;
    void plater.absoluteRotation(object, 0).then((degrees) => !stale && setRotation({ matrix, degrees }));
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
        onCommit={(axis, degrees) => void rotateInstance(target, radians(axis, degrees))} />
      {rotation?.matrix === matrix && (
        <Fields name="Rotate (absolute)" values={rotation.degrees} unit="°" disabled={slicing}
          onCommit={(axis, degrees) => void setInstanceRotation(target, axis, degrees)} />
      )}
    </Section>
  );
}
