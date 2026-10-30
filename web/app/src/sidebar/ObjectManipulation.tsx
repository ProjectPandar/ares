import { useState } from "react";
import { ObjectIcon } from "../shell/icons";
import { setInstancePosition } from "../state/plates";
import { useApp } from "../state/store";
import { Section } from "./Section";

const AXES = ["X", "Y", "Z"] as const;

/** A coordinate field committed on Enter or blur; external changes replace the text. */
function CoordinateField({ label, value, disabled, onCommit }: {
  label: string; value: number; disabled: boolean; onCommit: (value: number) => void;
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
      <input className="option-field" aria-label={`Position ${label}`} value={text} disabled={disabled} inputMode="decimal"
        onChange={(event) => setDraft({ shown, text: event.target.value })}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === "Enter") commit();
        }} />
      <span className="manipulation-unit">mm</span>
    </label>
  );
}

/**
 * The selected object's first instance in world coordinates, as GizmoObjectManipulation shows a
 * single full instance (`get_instance_offset`); an edit moves it (`change_position_value`).
 */
export function ObjectManipulation() {
  const scene = useApp((state) => state.scene);
  const plater = useApp((state) => state.plater);
  const selected = useApp((state) => state.selectedObject);
  const slicing = useApp((state) => state.slice.status === "slicing");
  const object = scene && plater ? scene.objects.findIndex(({ id }) => id === selected) : -1;
  if (object < 0) return null;
  const matrix = scene!.objects[object].instances[0].matrix;
  return (
    <Section icon={<ObjectIcon />} title="Object operations">
      <div className="manipulation" role="group" aria-label="Position">
        <span className="manipulation-title">Position</span>
        {AXES.map((label, axis) => (
          <CoordinateField key={label} label={label} value={matrix[12 + axis]} disabled={slicing}
            onCommit={(value) => void setInstancePosition({ object, instance: 0 }, axis as 0 | 1 | 2, value)} />
        ))}
      </div>
    </Section>
  );
}
