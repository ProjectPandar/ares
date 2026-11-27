import { useId, useRef, useState } from "react";
import "./splitAction.css";

export interface ToolbarAction {
  label: string;
  run: () => void;
  disabled?: boolean;
}

export function SplitAction({
  label,
  actions,
  primary = false,
  busy = false,
  title,
}: {
  label: string;
  actions: readonly [ToolbarAction, ...ToolbarAction[]];
  primary?: boolean;
  busy?: boolean;
  title?: string;
}) {
  const id = useId();
  const trigger = useRef<HTMLButtonElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const [selection, setSelection] = useState(actions[0].label);
  const [open, setOpen] = useState(false);
  const selected = actions.find((action) => action.label === selection) ?? actions[0];
  const optionsLabel = `${label} options`;
  const close = () => {
    popup.current!.hidePopover();
    trigger.current!.focus();
  };
  const toggle = () => {
    const menu = popup.current!;
    if (menu.matches(":popover-open")) {
      close();
      return;
    }
    const rect = trigger.current!.getBoundingClientRect();
    menu.style.right = `${window.innerWidth - rect.right}px`;
    const above = rect.top > window.innerHeight / 2;
    menu.style.top = above ? "auto" : `${rect.bottom + 6}px`;
    menu.style.bottom = above ? `${window.innerHeight - rect.top + 6}px` : "auto";
    menu.showPopover();
    menu.querySelector<HTMLButtonElement>('[aria-checked="true"]')!.focus();
  };
  return (
    <div className="action-split" role="group" aria-label={label}>
      <button
        type="button"
        className={`side-button action-main${primary ? " slice-action" : ""}`}
        disabled={selected.disabled}
        onClick={selected.run}
        title={title}
      >
        {busy ? "Slicing…" : selected.label}
      </button>
      <button
        type="button"
        ref={trigger}
        className={`side-button action-trigger${primary ? " slice-action" : ""}`}
        aria-label={optionsLabel}
        aria-haspopup="menu"
        aria-controls={id}
        aria-expanded={open}
        onClick={toggle}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          if (!popup.current!.matches(":popover-open")) toggle();
        }}
      />
      <div
        ref={popup}
        id={id}
        popover="auto"
        className="action-menu"
        role="menu"
        aria-label={optionsLabel}
        onToggle={({ newState }) => setOpen(newState === "open")}
        onKeyDown={(event) => {
          if (event.key === "Escape" || event.key === "Tab") {
            close();
            if (event.key === "Escape") event.preventDefault();
            return;
          }
          const options = [...event.currentTarget.querySelectorAll("button")];
          const index = options.findIndex((option) => option === document.activeElement);
          const last = options.length - 1;
          const next = {
            ArrowDown: index === last ? 0 : index + 1,
            ArrowUp: index <= 0 ? last : index - 1,
            Home: 0,
            End: last,
          }[event.key];
          if (next === undefined) return;
          event.preventDefault();
          options[next].focus();
        }}
      >
        {actions.map((action) => (
          <button
            key={action.label}
            type="button"
            role="menuitemradio"
            aria-checked={selected === action}
            tabIndex={-1}
            className="action-option"
            onClick={() => {
              setSelection(action.label);
              close();
            }}
          >
            {action.label}
          </button>
        ))}
      </div>
    </div>
  );
}
