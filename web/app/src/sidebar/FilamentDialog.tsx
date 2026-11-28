import { useEffect, useMemo, useRef, useState } from "react";
import { DialogHeader } from "../shell/DialogHeader";
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
function FilterColumn({
  title,
  placeholder,
  values,
  checked,
  onChange,
}: {
  title: string;
  placeholder: string;
  values: string[];
  checked: ReadonlySet<string>;
  onChange: (checked: ReadonlySet<string>) => void;
}) {
  const [search, setSearch] = useState("");
  const shown = values.filter((value) => matches(value, search));
  return (
    <section className="frame" role="group" aria-label={title}>
      <header className="frame-head">
        <h2>{title}</h2>
      </header>
      <div className="frame-content">
        <input
          type="search"
          className="search"
          aria-label={`Search ${title}`}
          placeholder={placeholder}
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
        <div className="check-list">
          <label className="check">
            <input
              type="checkbox"
              checked={values.every((value) => checked.has(value))}
              onChange={(event) => onChange(event.target.checked ? new Set(values) : new Set())}
            />
            <span>all</span>
          </label>
          {shown.map((value) => (
            <label key={value} className="check">
              <input
                type="checkbox"
                checked={checked.has(value)}
                onChange={(event) => onChange(toggled(checked, value, event.target.checked))}
              />
              <span>{value}</span>
            </label>
          ))}
          {shown.length === 0 && <div className="side-empty">No items</div>}
        </div>
      </div>
    </section>
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
      <div className="dialog-body">
        <div className="catalog">
          <FilterColumn
            title="Filament type"
            placeholder="Search types"
            values={model.types}
            checked={types}
            onChange={setTypes}
          />
          <FilterColumn
            title="Vendor"
            placeholder="Search vendors"
            values={model.vendors}
            checked={vendors}
            onChange={setVendors}
          />
          <section className="frame" role="group" aria-label="Filaments">
            <header className="frame-head">
              <h2>Filaments</h2>
              <span className="catalog-actions">
                <button className="button" onClick={() => setShown(true)}>
                  All
                </button>
                <button className="button" onClick={() => setShown(false)}>
                  Clear all
                </button>
              </span>
            </header>
            <div className="frame-content">
              <input
                type="search"
                className="search"
                aria-label="Filter filaments"
                placeholder="Filter items"
                value={filter}
                onChange={(event) => setFilter(event.target.value)}
              />
              <div className="check-list">
                {shown.map((item) => (
                  <label key={item.key} className="check" title={item.names.join("\n")}>
                    <input
                      type="checkbox"
                      aria-label={item.short}
                      checked={checked.has(item.key)}
                      onChange={(event) => setChecked(toggled(checked, item.key, event.target.checked))}
                    />
                    <span>{item.short}</span>
                    <span className="check-detail">
                      {item.vendor} · {item.type}
                    </span>
                  </label>
                ))}
                {shown.length === 0 && <div className="side-empty">No items</div>}
              </div>
            </div>
          </section>
        </div>
      </div>
      <footer className="dialog-footer">
        <span className="summary" data-testid="filament-count">
          {checked.size} / {model.items.length}
          {shown.length < model.items.length ? ` [${shown.length}]` : ""}
        </span>
        <button className="button primary" onClick={() => onDone(checked)}>
          Done
        </button>
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
    <dialog ref={dialog} className="dialog wide sheet" aria-labelledby="filaments-title" onClose={onClose}>
      <DialogHeader
        id="filaments-title"
        title="Filament selection"
        description={printer}
        close={() => dialog.current!.close()}
      />
      {model ? (
        <Columns
          model={model}
          onDone={(checked) => {
            setInstalledFilaments(installedAfter(useApp.getState().installedFilaments, model.items, checked));
            dialog.current!.close();
          }}
        />
      ) : (
        <div className="dialog-body">
          <div className="side-empty">Loading filaments…</div>
        </div>
      )}
    </dialog>
  );
}
