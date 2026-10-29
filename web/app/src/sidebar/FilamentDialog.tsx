import { useEffect, useMemo, useRef, useState } from "react";
import { dialogFilaments, setInstalledFilaments } from "../state/presets";
import { useApp } from "../state/store";
import { checkedItems, type DialogModel, dialogModel, installedAfter, itemShown, matches } from "./filamentDialog";

const toggled = (set: ReadonlySet<string>, value: string, on: boolean) => {
  const next = new Set(set);
  if (on) next.add(value);
  else next.delete(value);
  return next;
};

/** A searchable checkbox column with its "all" box (guide/23/index.html `cbr-column`). */
function FilterColumn({ title, values, checked, onChange }: {
  title: string;
  values: string[];
  checked: ReadonlySet<string>;
  onChange: (checked: ReadonlySet<string>) => void;
}) {
  const [search, setSearch] = useState("");
  const shown = values.filter((value) => matches(value, search));
  return (
    <div className="filament-column" role="group" aria-label={title}>
      <input type="search" className="filament-column-search" aria-label={`Search ${title}`} placeholder={title}
        value={search} onChange={(event) => setSearch(event.target.value)} />
      <div className="filament-column-list">
        <label className="vendor-item">
          <input type="checkbox" checked={values.every((value) => checked.has(value))}
            onChange={(event) => onChange(event.target.checked ? new Set(values) : new Set())} />
          all
        </label>
        {shown.map((value) => (
          <label key={value} className="vendor-item">
            <input type="checkbox" checked={checked.has(value)} onChange={(event) => onChange(toggled(checked, value, event.target.checked))} />
            {value}
          </label>
        ))}
        {shown.length === 0 && <div className="side-empty">No items</div>}
      </div>
    </div>
  );
}

function Columns({ model, onDone }: { model: DialogModel; onDone: (checked: ReadonlySet<string>) => void }) {
  const installed = useApp((state) => state.installedFilaments);
  const [types, setTypes] = useState<ReadonlySet<string>>(() => new Set(model.types));
  const [vendors, setVendors] = useState<ReadonlySet<string>>(() => new Set(model.vendors));
  const [checked, setChecked] = useState<ReadonlySet<string>>(() => checkedItems(model.items, installed));
  const [filter, setFilter] = useState("");
  const shown = model.items.filter((item) => itemShown(item, types, vendors, filter, checked.has(item.key)));
  // SelectAllFilament (common.js:394-407): "all" and "Clear all" act on the listed items.
  const setShown = (on: boolean) => setChecked(shown.reduce((next, item) => toggled(next, item.key, on), checked));
  return (
    <>
      <div className="filament-columns">
        <FilterColumn title="Filament type" values={model.types} checked={types} onChange={setTypes} />
        <FilterColumn title="Vendor" values={model.vendors} checked={vendors} onChange={setVendors} />
        <div className="filament-column filament-items" role="group" aria-label="Filaments">
          <div className="filament-items-bar">
            <input type="search" className="filament-column-search" aria-label="Filter filaments" placeholder="Filter items"
              value={filter} onChange={(event) => setFilter(event.target.value)} />
            {/* UpdateStats (common.js:379-392): checked / all [listed]. */}
            <span className="filament-count" data-testid="filament-count">
              {checked.size} / {model.items.length}{shown.length < model.items.length ? ` [${shown.length}]` : ""}
            </span>
            <button className="side-button" onClick={() => setShown(true)}>all</button>
            <button className="side-button" onClick={() => setShown(false)}>Clear all</button>
          </div>
          <div className="filament-column-list">
            {shown.map((item) => (
              <label key={item.key} className="vendor-item" title={item.names.join("\n")}>
                <input type="checkbox" checked={checked.has(item.key)}
                  onChange={(event) => setChecked(toggled(checked, item.key, event.target.checked))} />
                {item.short}
              </label>
            ))}
            {shown.length === 0 && <div className="side-empty">No items</div>}
          </div>
        </div>
      </div>
      <footer className="vendor-footer">
        <button className="side-button" onClick={() => onDone(checked)}>Done</button>
      </footer>
    </>
  );
}

/** "Add/Remove filaments" (PresetComboBoxes.cpp:1460-1461, LABEL_ITEM_WIZARD_FILAMENTS): the filament selection dialog. */
export function FilamentDialog({ printer, onClose }: { printer: string; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [filaments, setFilaments] = useState<Awaited<ReturnType<typeof dialogFilaments>> | null>(null);
  useEffect(() => {
    dialog.current!.showModal();
    void dialogFilaments(printer).then(setFilaments);
  }, [printer]);
  const model = useMemo(() => filaments && dialogModel(filaments), [filaments]);
  return (
    <dialog ref={dialog} className="shortcuts-dialog filament-dialog" aria-labelledby="filaments-title" onClose={onClose}>
      <header className="shortcuts-header">
        <h2 id="filaments-title">Filament selection</h2>
        <button className="bar-button" aria-label="Close" onClick={() => dialog.current!.close()}>{"\u2715"}</button>
      </header>
      {model ? (
        <Columns model={model} onDone={(checked) => {
          setInstalledFilaments(installedAfter(useApp.getState().installedFilaments, model.items, checked));
          dialog.current!.close();
        }} />
      ) : <div className="side-empty">Loading filaments…</div>}
    </dialog>
  );
}
