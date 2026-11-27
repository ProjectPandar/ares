import type { FilamentSlotEdit, FilamentSlots } from "@ares-slicer/wasm";
import { useEffect, useId, useRef, useState } from "react";
import "./filamentAssignment.css";

export function FilamentSlotMenu({
  slot,
  slots,
  disabled,
  edit,
  openPreset,
}: {
  slot: number;
  slots: FilamentSlots["slots"];
  disabled: boolean;
  edit: (edit: FilamentSlotEdit) => void;
  openPreset: () => void;
}) {
  const id = useId();
  const popup = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const [merging, setMerging] = useState(false);
  const [mergeTarget, setMergeTarget] = useState<number>();
  const confirmation = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (mergeTarget !== undefined) confirmation.current!.showModal();
  }, [mergeTarget]);
  const close = () => {
    popup.current!.hidePopover();
    trigger.current!.focus();
  };
  return (
    <>
      <button
        type="button"
        className="preset-edit"
        ref={trigger}
        aria-label={`Filament ${slot} actions`}
        disabled={disabled}
        aria-haspopup="menu"
        aria-controls={id}
        aria-expanded={open}
        onClick={() => {
          const menu = popup.current!;
          if (menu.matches(":popover-open")) {
            close();
            return;
          }
          const button = trigger.current!.getBoundingClientRect();
          menu.style.left = `${button.right}px`;
          menu.style.top = `${button.bottom + 2}px`;
          menu.style.transform = "translateX(-100%)";
          menu.showPopover();
          menu.querySelector("button")?.focus();
        }}
      >
        <span aria-hidden="true">⋯</span>
      </button>
      <div
        id={id}
        ref={popup}
        popover="auto"
        className="context-menu filament-cell-menu"
        role="menu"
        aria-label={`Filament ${slot} actions`}
        onToggle={({ newState }) => {
          setOpen(newState === "open");
          if (newState === "closed") setMerging(false);
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            close();
          }
          if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
          event.preventDefault();
          const items = [...event.currentTarget.querySelectorAll("button")];
          const focused = items.indexOf(document.activeElement as HTMLButtonElement);
          items[(items.length + focused + (event.key === "ArrowDown" ? 1 : -1)) % items.length]?.focus();
        }}
      >
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            close();
            openPreset();
          }}
        >
          Edit
        </button>
        <button
          type="button"
          role="menuitem"
          aria-haspopup="menu"
          aria-expanded={merging}
          onClick={() => setMerging(!merging)}
        >
          Merge with <span aria-hidden="true">›</span>
        </button>
        {merging && (
          <div role="menu" aria-label="Merge with">
            {slots.map(
              ({ preset, colour, components }, index) =>
                index + 1 !== slot && (
                  <button
                    key={index}
                    type="button"
                    role="menuitem"
                    aria-label={`Filament ${index + 1}: ${preset}`}
                    onClick={() => {
                      close();
                      const target = index + 1;
                      if (components.includes(slot)) setMergeTarget(target);
                      else
                        edit({
                          kind: "merge",
                          source: slot,
                          target,
                          confirmed: false,
                        });
                    }}
                  >
                    <span className="filament-swatch" aria-hidden="true" style={{ background: colour }}>
                      {index + 1}
                    </span>
                    {preset || `Filament ${index + 1}`}
                  </button>
                ),
            )}
          </div>
        )}
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            close();
            edit({ kind: "delete", slot });
          }}
        >
          Delete
        </button>
      </div>
      {mergeTarget !== undefined && (
        <dialog
          ref={confirmation}
          className="shortcuts-dialog"
          aria-labelledby={`${id}-merge-title`}
          role="alertdialog"
          onClose={() => {
            setMergeTarget(undefined);
            trigger.current?.focus();
          }}
        >
          <header className="shortcuts-header">
            <h2 id={`${id}-merge-title`}>Merge filament?</h2>
          </header>
          <div className="dialog-body">
            <p className="dialog-notice warning">
              The target mixed filament uses this filament as a component. Merging may invalidate the mix.
            </p>
          </div>
          <footer className="vendor-footer">
            <button className="side-button" onClick={() => confirmation.current!.close()}>
              Cancel
            </button>
            <button
              className="side-button dialog-danger"
              onClick={() => {
                confirmation.current!.close();
                edit({
                  kind: "merge",
                  source: slot,
                  target: mergeTarget,
                  confirmed: true,
                });
              }}
            >
              Merge
            </button>
          </footer>
        </dialog>
      )}
    </>
  );
}
