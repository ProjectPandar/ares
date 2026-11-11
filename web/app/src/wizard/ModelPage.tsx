import type { GuideModel } from "@ares-slicer/wasm";
import { useEffect, useRef, useState } from "react";

/** CreateVendorBlock's banner name (guide/21/common.js:269-277). */
const vendorTitle = (vendor: string) =>
  vendor === "BBL" ? "Bambu Lab" : vendor === "Custom" ? "Custom Printer" : vendor === "Other" ? "Orca colosseum" : vendor;

/** CreatePrinterBlock's card name (common.js:292-309): the model name without its vendor prefix. */
function modelTitle({ vendor, name }: GuideModel) {
  const prefix = vendor === "BBL" ? "Bambu Lab" : vendor;
  if (vendor === "Custom") return name.split(" ")[1];
  if (name.toLowerCase().startsWith(prefix.toLowerCase())) return name.slice(prefix.length);
  if (vendor === "MagicMaker" && name.startsWith("MM")) return name.slice("MM".length);
  if (vendor === "OrcaArena") return name.slice("Orca Arena".length);
  if (vendor === "RolohaunDesign" && name.startsWith("Rolohaun")) return name.slice("Rolohaun".length);
  return name;
}

/** HandleModelList's order (common.js:77-82): by vendor, the Custom printers first. */
export function sortedModels(models: readonly GuideModel[]): GuideModel[] {
  const sorted = [...models].sort((a, b) => a.vendor.localeCompare(b.vendor));
  return [...sorted.filter(({ vendor }) => vendor === "Custom"), ...sorted.filter(({ vendor }) => vendor !== "Custom")];
}

/** FilterModelList's test (common.js:204-213): every whitespace-separated keyword is in the name or vendor. */
function matchesSearch({ name, vendor }: GuideModel, search: string) {
  const text = `${name}\0${vendor}`.toLowerCase();
  return (search.toLowerCase().match(/\S+/g) ?? []).every((keyword) => text.includes(keyword));
}

function VendorBlock({ vendor, models, checked, onChange }: {
  vendor: string;
  models: GuideModel[];
  checked: ReadonlySet<string>;
  onChange: (checked: ReadonlySet<string>) => void;
}) {
  const box = useRef<HTMLInputElement>(null);
  const count = models.filter(({ model }) => checked.has(model)).length;
  // UpdateVendorCheckbox (common.js:360-370): checked when all are, indeterminate when some are.
  useEffect(() => {
    box.current!.indeterminate = count > 0 && count < models.length;
  }, [count, models.length]);
  const set = (model: string, on: boolean, next: Set<string>) => {
    if (on) next.add(model);
    else next.delete(model);
    return next;
  };
  // ChooseVendor (common.js:348-357): checks the listed models unless all are checked.
  const chooseVendor = () => {
    const on = count < models.length;
    onChange(models.reduce((next, { model }) => set(model, on, next), new Set(checked)));
  };
  return (
    <section className="wizard-vendor" data-vendor={vendor} aria-label={vendorTitle(vendor)}>
      <header className="wizard-vendor-banner">
        <h3>{vendorTitle(vendor)}</h3>
        <label className="wizard-vendor-check">
          <span className="wizard-count">{count} / {models.length}</span>
          <input ref={box} type="checkbox" aria-label={`All ${vendorTitle(vendor)} models`}
            checked={count > 0 && count === models.length} onChange={chooseVendor} />
        </label>
      </header>
      <div className="wizard-models">
        {models.map((model) => (
          <label key={model.model} className="wizard-model" title={model.name}>
            <img className="wizard-cover" src={`${import.meta.env.BASE_URL}printer-dummy.png`} alt="" />
            <span className="wizard-nozzle">Nozzle <span>{model.variants.join(" · ")}</span></span>
            <span className="wizard-model-name">
              <input type="checkbox" aria-label={model.name} checked={checked.has(model.model)}
                onChange={(event) => onChange(set(model.model, event.target.checked, new Set(checked)))} />
              {modelTitle(model)}
            </span>
          </label>
        ))}
      </div>
    </section>
  );
}

/** Printer Selection, guide pages 21 and 24: the vendor sidebar, the search and one block per vendor. */
export function ModelPage({ models, checked, onChange }: {
  models: readonly GuideModel[];
  checked: ReadonlySet<string>;
  onChange: (checked: ReadonlySet<string>) => void;
}) {
  const [search, setSearch] = useState("");
  const content = useRef<HTMLDivElement>(null);
  const byVendor = new Map<string, GuideModel[]>();
  for (const model of models) {
    if (matchesSearch(model, search)) byVendor.set(model.vendor, [...byVendor.get(model.vendor) ?? [], model]);
  }
  // scrollToVendor (common.js:328-334).
  const scrollTo = (vendor: string) =>
    content.current!.querySelector<HTMLElement>(`[data-vendor="${CSS.escape(vendor)}"]`)!.scrollIntoView({ behavior: "smooth" });
  return (
    <div className="wizard-printers">
      <nav className="wizard-sidebar" aria-label="Vendors">
        <div className="wizard-sidebar-title">Vendors</div>
        {[...byVendor.keys()].map((vendor) => (
          <button key={vendor} className="wizard-sidebar-item" onClick={() => scrollTo(vendor)}>
            {vendor === "Custom" ? "Custom" : vendor}
          </button>
        ))}
      </nav>
      <div className="wizard-content">
        <input type="search" className="filament-column-search wizard-search" aria-label="Search printers" placeholder="Search"
          value={search} onChange={(event) => setSearch(event.target.value)} />
        <div ref={content} className="wizard-vendors">
          {[...byVendor].map(([vendor, listed]) => (
            <VendorBlock key={vendor} vendor={vendor} models={listed} checked={checked} onChange={onChange} />
          ))}
        </div>
      </div>
    </div>
  );
}
