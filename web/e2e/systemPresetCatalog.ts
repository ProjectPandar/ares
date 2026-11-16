import { loadVendor, vendors } from "@ares-slicer/profiles";

/** The installed bundle, using the package's existing vendor-envelope transport. */
export async function systemPresetCatalog(): Promise<string> {
  const installed = vendors.filter(({ name }) => name);
  installed.sort((a, b) => Number(b.id === "OrcaFilamentLibrary") - Number(a.id === "OrcaFilamentLibrary") || a.id.localeCompare(b.id));
  return JSON.stringify({ vendors: await Promise.all(installed.map(({ id }) => loadVendor(id))), imports: [] });
}
