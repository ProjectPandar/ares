// A third-party app listing vendors eagerly and loading one vendor's presets on demand.
import { loadVendor, vendors } from "@ares-slicer/profiles";

const list = (testId: string, items: string[]) =>
  document.querySelector(`[data-testid=${testId}]`)!.replaceChildren(
    ...items.map((item) => Object.assign(document.createElement("li"), { textContent: item })),
  );

list("vendors", vendors.map((vendor) => vendor.id));
document.querySelector("button")!.addEventListener("click", async () => {
  const { index, files } = await loadVendor("Snapmaker");
  list("printers", index.machine_list.map((entry) => files[entry.sub_path].name));
});
