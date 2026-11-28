import { useEffect, useRef, useState } from "react";
import { DialogHeader } from "../shell/DialogHeader";
import { setNumberOfCopies } from "../state/plates";

/** `GetNumberFromUser` of `Plater::set_number_of_copies`: 0 to 1000 copies (Plater.cpp:19394-19395). */
const MAX_COPIES = 1000;

/**
 * "Set number of instances" (Plater::set_number_of_copies, Plater.cpp:19386-19406): the count of the
 * object at `object`, preset to its current one; OK adds or removes instances to reach it.
 */
export function CopiesDialog({ object, count, onClose }: { object: number; count: number; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [value, setValue] = useState(String(count));
  useEffect(() => dialog.current!.showModal(), []);
  const num = Number(value);
  const valid = value.trim() !== "" && Number.isInteger(num) && num >= 0 && num <= MAX_COPIES;
  return (
    <dialog ref={dialog} className="dialog" aria-labelledby="copies-title" onClose={onClose}>
      <DialogHeader id="copies-title" title="Copies of the selected object" close={() => dialog.current!.close()} />
      <div className="dialog-body">
        <label className="field">
          <span>Number of copies:</span>
          <input
            type="number"
            min={0}
            max={MAX_COPIES}
            step={1}
            value={value}
            onChange={(event) => setValue(event.target.value)}
          />
        </label>
      </div>
      <footer className="dialog-footer">
        <button className="button" onClick={() => dialog.current!.close()}>
          Cancel
        </button>
        <button
          className="button primary"
          disabled={!valid}
          onClick={() => {
            void setNumberOfCopies(object, num);
            dialog.current!.close();
          }}
        >
          OK
        </button>
      </footer>
    </dialog>
  );
}
