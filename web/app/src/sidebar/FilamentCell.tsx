import type { FilamentTarget } from "@ares-slicer/wasm";
import { useId, useRef, useState, type KeyboardEvent, type MouseEvent } from "react";
import { assignFilament, filamentPalette } from "../state/filamentAssignment";
import { useApp } from "../state/store";
import { filamentNames } from "./options";
import "./filamentAssignment.css";

const slotLabel = (slot: number) => slot ? `Filament ${slot}` : "Default";

/** ObjectList's Fila. column editor (GUI_ObjectList.cpp:459–482). */
export function FilamentCell({ name, target, slot, modifier = false }: {
  name: string; target: FilamentTarget; slot: number; modifier?: boolean;
}) {
  const settings = useApp((state) => state.scene?.settings ?? null);
  const selection = useApp((state) => state.presetSelection);
  const available = useApp((state) => Boolean(state.plater) && !state.arranging && state.slice.status !== "slicing");
  const trigger = useRef<HTMLButtonElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const menuId = useId();
  const count = selection?.filaments.length ?? filamentNames(settings).length;
  if (count < 2) return null;
  const palette = filamentPalette(selection, settings);
  const choices = Array.from({ length: count + Number(modifier) }, (_, index) => index + Number(!modifier));
  const background = (value: number) => palette[value - 1] ?? "#888";

  const close = () => {
    popup.current!.hidePopover();
    trigger.current!.focus();
  };
  const activate = (event: MouseEvent<HTMLButtonElement>) => {
    event.stopPropagation();
    const menu = popup.current!;
    if (menu.matches(":popover-open")) {
      close();
      return;
    }
    const button = event.currentTarget;
    const cell = button.getBoundingClientRect();
    button.focus();
    menu.showPopover();
    const { width, height } = menu.getBoundingClientRect();
    menu.style.left = `${Math.max(4, Math.min(cell.right, window.innerWidth - 4) - width)}px`;
    menu.style.top = `${cell.bottom + 2 + height <= window.innerHeight - 4 ? cell.bottom + 2 : Math.max(4, cell.top - 2 - height)}px`;
    const active = menu.querySelector<HTMLButtonElement>('[aria-checked="true"]') ?? menu.querySelector("button")!;
    active.focus();
  };
  const navigate = (event: KeyboardEvent<HTMLDivElement>) => {
    switch (event.key) {
      case "Escape":
        event.preventDefault();
        event.stopPropagation();
        close();
        break;
      case "ArrowDown": case "ArrowUp": case "Home": case "End": {
        event.preventDefault();
        const items = Array.from(event.currentTarget.querySelectorAll("button"));
        const index = items.indexOf(document.activeElement as HTMLButtonElement);
        const next = event.key === "Home" ? 0 : event.key === "End" ? items.length - 1
          : (index + items.length + (event.key === "ArrowDown" ? 1 : -1)) % items.length;
        items[next].focus();
        break;
      }
    }
  };
  return <>
    <button type="button" ref={trigger} className="filament-cell" disabled={!available} onClick={activate}
      aria-label={`Edit filament for ${name}`} aria-haspopup="menu" aria-expanded={open} aria-controls={menuId}>
      <span className="filament-swatch" role="img" aria-label={slotLabel(slot)} title={slotLabel(slot)}
        style={{ background: background(slot) }}>{slot || "–"}</span>
    </button>
    <div id={menuId} ref={popup} popover="auto" className="context-menu filament-cell-menu"
      role="menu" aria-label={`Filament for ${name}`} onKeyDown={navigate} onToggle={(event) => setOpen(event.newState === "open")}>
      {choices.map((value) => <button key={value} type="button" role="menuitemradio" aria-checked={slot === value}
        aria-label={slotLabel(value)} tabIndex={-1} disabled={!available} onClick={(event) => {
          event.stopPropagation();
          close();
          void assignFilament({ cell: target }, value);
        }}>
        {value ? <span className="filament-swatch" aria-hidden="true" style={{ background: background(value) }}>{value}</span> : "Default"}
      </button>)}
    </div>
  </>;
}
