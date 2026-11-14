# Web: load the preset bundle once per catalog

## Problem

Opening a project with the plater's Always selection builds the process, printer and filament
collections from the whole catalog on every call. Each preset is composed, and its variant vectors
are padded on the way. With every Bambu Lab printer installed, one such call takes about 7.5 s in
Node on one core. Of that, about 5.5 s is the load. An H2D project then misses the 5 s budget for
"Ready to slice".

## Upstream reference

- `PresetBundle::load_presets` (PresetBundle.cpp:881) runs at startup (GUI_App.cpp:3723) and
  after the configuration wizard applies its choices (ConfigWizard.cpp:2597).
- `PresetCollection::load_presets` (Preset.cpp:2032) loads each preset once. Variant padding
  happens there: `extend_default_config_length` at Preset.cpp:1930 and :1944.
- `PresetBundle::load_config_model` (Plater.cpp:9623) selects from these loaded collections when
  a project opens. It adds the project's presets to them and does not load the bundle again.

## Destination

- `crates/ares-core/src/profiles/project_selection/loaded.rs`: `LoadedCatalog`. It holds the
  parsed catalog and its three collections. The collections are loaded on first use, then kept.
  - The most recent catalog is kept, keyed by its JSON text.
  - `load_preset_bundle(catalog_json)` loads ahead of a project.
  - A project selection copies the loaded collection before it adds project presets, so one
    project's presets never reach the next.
- `ProjectPresetRefresh` takes the `LoadedCatalog`. The CLI policy only reads its catalog and
  never loads the collections.
- `crates/ares-wasm`: `loadPresetBundle(catalogJson)`.
- `@ares-slicer/wasm`: a worker request and `AresSlicer.loadPresetBundle()`.
- `web/app/src/state/catalog.ts`: `syncCatalog` calls it whenever the installed vendors or the
  imports change. That covers startup and the wizard's Confirm, which waits for the sync. The
  catalog sync moves here from `presets.ts`, which had grown past 400 lines.

## Included

- One load per catalog in a worker session.
- The preload at startup and after the wizard.
- A project open that follows a preload reuses it.

## Deferred

- `load_presets` substitutions and user preset folders (the catalog has none).
- Keeping more than one catalog loaded.

## Key

The slicing calls receive the catalog as JSON text, and every call already copies that text into
the worker. The cache compares the text it holds with the text it receives. Both have to be
copied whole anyway, so the comparison adds one pass over bytes already in memory. In Node,
with every Bambu Lab printer installed, the catalog is 4.8 MB. A repeat preload with the same
text takes 7-40 ms, transfer included, against 10.9 s for the first load.

## Failure modes and how each is ruled out

1. **A project's presets leak into the next project.** Each selection works on a copy of the
   loaded collection.
2. **A stale catalog is used after the vendors or imports change.** The key is the whole catalog
   text. Any change in vendors or imports changes it.
3. **Two loads run at once.** The worker handles one request at a time, and the load is
   synchronous. A project open that arrives during the preload waits for it, then finds it
   loaded.
4. **A failed load is kept.** The collections are kept only after all three load.
5. **The CLI pays for collections it never uses.** Its policy reads only the parsed catalog.

## Tests

`web/e2e/presetBundleLoad.spec.ts`, desktop project:
- installs every Bambu Lab printer in the wizard;
- opens the H2D project used by `dualExtruder.spec.ts`;
- expects "Ready to slice" within the default 5 s;
- writes the time from opening the file to "Ready to slice" to `result.json`.

Neighbouring specs, run before and after the change:
- `dualExtruder.spec.ts`;
- `presets.spec.ts`;
- `import.spec.ts`.
