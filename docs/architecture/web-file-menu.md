# Web File menu

## Source boundary

This slice ports OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) GUI behaviour.
Paths are relative to `src/slic3r/GUI/`.

- `MainFrame::init_menubar_as_editor` builds the File menu (`MainFrame.cpp:3030-3183`, trailing
  items `:3743-3779`) and hands it to the top bar (`m_topbar->SetFileMenu(fileMenu)`, `:3619`).
- `BBLTopbar` shows it as the "File" tool with the `topbar_file` icon at the left of the title bar
  (`BBLTopbar.cpp:290-292`) and pops it up under that tool (`OnFileToolItem`, `:647-659`).
- Shortcuts come from `Shortcuts.cpp:60-70` and run through the MainFrame key handler
  (`MainFrame.cpp:905-930`).
- Enable predicates: `can_start_new_project`, `can_open_project`, `can_add_models`,
  `can_save`, `can_save_as`, `can_export_model`, `can_export_gcode`, `can_export_toolpaths`
  (`MainFrame.cpp:1957-2073`).

The item order is the one OrcaSlicer renders on macOS, which the user supplied as the reference:
New Window | New Project | Open Project… | Recent files › | Save Project | Save Project as… |
separator | Import › | Export › | separator | Sync Presets | separator | Preset Bundle.
On macOS, New Window (`:3033-3037`) and the File-menu Preset Bundle and Sync Presets
(`:3741-3771`, the macOS branch) are compiled in. Quit moves to the application menu, and Publish 3MF
(`:3096-3108`) does not appear. Open Speed Dial (`:3744-3746`) and Plugins (`:3775-3777`) are
not shown either. The web uses this one order on every platform.

## Ares today

`web/app/src/shell/TopBar.tsx`:
- The title bar shows `Ares | Import | Import Configs` on the left.
  - Import opens the hidden `import-input` (`.3mf,.stl`, multiple) and calls `importFiles`.
  - Import Configs opens `preset-import-input` and calls `importPresetFiles`.
- The TabBar has the side buttons Slice plate, Slice all, Export G-code file and Export all
  (`exportGcode`, `exportAllGcode` in `state/actions.ts`).
- The Help menu (`HelpMenu`) is the only drop-down menu.

`Shell.tsx` binds Ctrl+I (import), Ctrl+R (slice) and Ctrl+G (`exportGcode`). Ctrl is ⌘ on
macOS (`keys.ts`).

## Destination

- `TitleBar` replaces the Import and Import Configs buttons with one "File" bar button (the
  `topbar_file` icon and the label File), placed where the buttons were, after the brand.
- The menu is a new `shell/FileMenu.tsx`. It follows the `HelpMenu` pattern: closes on a pick,
  on Escape and on a press outside.
- Import and Export are submenus.
- The hidden file inputs stay in the title bar with their test ids, accepted types and
  handlers. Specs that set files on them directly are unchanged.
- Upstream items the web cannot perform yet stay in the menu, disabled, with their reason shown
  as a second line under the label. The same text is the item's accessible description.
  Clicking a disabled item does nothing and keeps the menu open. No item is a silent no-op.
- Accelerators are shown at the right of a row only when that key runs that row's action in the
  web. They use the platform prefix of `keys.ts` (`⌘+` on macOS, `Ctrl+` elsewhere).

## Item by item

### File

| Item | Upstream | Web |
|---|---|---|
| New Window | `start_new_slicer` (`:3033-3037`) | Enabled. Opens the app's URL in a new browser tab, a second instance that shares the stored app config as a second OrcaSlicer process shares AppConfig. |
| New Project | `Plater::new_project` (`Plater.cpp:16064`); `can_start_new_project` | Enabled unless a slice is running. See below. |
| Open Project… | `Plater::load_project`; `can_open_project` | Enabled unless a slice is running or a file is loading. Opens a picker for `.3mf` and loads it with the existing `importFile`, which replaces the current project as `load_project` does. Shortcut Ctrl+O. |
| Recent files › | `m_recent_projects` (`:3054-3074`), enabled only when the history is not empty | Disabled: "Recent files are not kept in the browser yet." A browser has no file paths to reopen; a history would have to keep the project bytes per viewer (IndexedDB). This is deferred to its own slice. |
| Save Project | `save_project()`; `can_save` | Disabled: "Needs the 3MF project writer." Enabled by the 3MF export slice ([web-export-3mf.md](web-export-3mf.md)). |
| Save Project as… | `save_project(true)`; `can_save_as` | Disabled with the same reason. Enabled by the 3MF export slice. |
| Import › | submenu `:3110-3132` | See Import. |
| Export › | submenu `:3135-3175` | See Export. |
| Sync Presets | `restart_sync_user_preset`, enabled only when logged in (`:3755-3771`) | Disabled: "Cloud preset sync is not available in the browser." The web has no cloud account. This is the state upstream shows when logged out. |
| Preset Bundle | `open_presetbundledialog` (`:3748-3753`) | Disabled: "The preset bundle dialog is not available yet." Importing a bundle file stays under Import › Import Configs. |

New Project resets the model part of the store to its start state:
- no file, scene, plater session, plate slices, slice result or arrange state;
- the plater session is freed;
- the Prepare tab is shown, as `new_project` selects `TAB_ID_PREPARE`.

The installed printers, the selected presets and their sidebar edits stay. This is upstream's
"Keep" answer to the modified-presets question (`check_and_keep_current_preset_changes`).

When a model is loaded, the web first asks "Discard the current project?" with Discard and
Cancel. This is the `close_with_confirm` step, without its Save button: there is no project
writer yet. The 3MF export slice adds Save to this dialog.

### Import ›

| Item | Upstream | Web |
|---|---|---|
| Import 3MF/STL… | "Import 3MF/STL/STEP/SVG/OBJ/AMF…" `Plater::add_file`; `can_add_models`; Ctrl+I | Enabled when the engine is ready and no file is loading. Opens `import-input` and runs the existing `importFiles`, which keeps its accepted extensions. The label names only the formats the web reads, as the shortcuts dialog already does. Ctrl+I stays bound. |
| Import ZIP Archive… | `import_zip_archive` | Disabled: "ZIP archives cannot be imported yet." |
| Import Configs… | `load_config_file`, always enabled | Enabled when the engine is ready. Opens `preset-import-input` and runs the existing `importPresetFiles`. |

Import still replaces the current model, as the web's import does today. Upstream `add_file`
adds the file to the current plate. That difference already exists and is not changed here.

### Export ›

| Item | Upstream | Web |
|---|---|---|
| Export all objects as one STL… | `export_stl()`; `can_export_model` | Disabled: "STL export is not available yet." |
| Export all objects as STLs… | `export_stl(false, false, true)` | Disabled, same reason. |
| Export all objects as one DRC… | `export_stl(…, FT_DRC)` | Disabled: "DRC export is not available yet." |
| Export all objects as DRCs… | `export_stl(…, true, FT_DRC)` | Disabled, same reason. |
| Export Generic 3MF… | `export_core_3mf` | Disabled: "Needs the 3MF project writer." The 3MF export slice keeps it disabled; see that document. |
| Export plate sliced file… | `EVT_GLTOOLBAR_EXPORT_SLICED_FILE`; `can_export_gcode`; Ctrl+G | Disabled: "Needs the 3MF project writer." Enabled by the 3MF export slice. |
| Export all plate sliced file… | `EVT_GLTOOLBAR_EXPORT_ALL_SLICED_FILE` | Disabled, same reason. The 3MF export slice keeps it disabled. |
| Export G-code… | `export_gcode(false)`; `can_export_gcode` | Enabled when the current plate's slice is done (the side button's condition). Runs the existing `exportGcode`: the same bytes and file name as the button. |
| Export toolpaths as OBJ… | `export_toolpaths_to_obj`; `can_export_toolpaths` | Disabled: "Toolpath OBJ export is not available yet." |
| Export Preset Bundle… | `export_config` | Disabled: "Preset bundle export is not available yet." |

### Not shown

- Quit (`:3178-3181`): a browser tab is closed by the browser.
- Publish 3MF, Open Speed Dial and Plugins: absent from the reference menu.

## Side buttons and shortcuts

- The TabBar keeps Export G-code file and Export all. Upstream has both the menu item and the
  SideButton print actions (`available_print_actions`, `:2225-2266`).
- Export all, one G-code file per sliced plate, has no upstream menu item, so it stays a side
  button only.
- Ctrl+O is new and opens the Open Project picker. Shell's key handler intercepts it before the
  browser does.
- Ctrl+N is reserved by browsers and cannot be intercepted, so New Project shows no
  accelerator.
- Ctrl+S and Ctrl+Shift+S are bound by the 3MF export slice when it enables the Save items.
- Ctrl+G keeps running `exportGcode` in this slice. The shortcuts dialog's wording ("Export
  plate sliced file") stays. The 3MF export slice rebinds Ctrl+G to Export plate sliced file
  (`Shortcuts.cpp:70`) and shows it on that row.
- The shortcuts dialog adds Ctrl+O "Open Project" (upstream group "Global shortcuts").

## Keyboard and accessibility

- The File button has `aria-haspopup="menu"` and `aria-expanded`.
- Enter, Space or ArrowDown opens the menu and focuses its first enabled item.
- ArrowUp and ArrowDown move between items. Disabled items are focusable and announce their
  reason, but do not act.
- ArrowRight, Enter or Space on Import or Export opens the submenu and focuses its first
  enabled item. ArrowLeft or Escape closes the submenu and returns focus to its parent item.
- Escape on the top menu closes it and returns focus to the File button.
- A press outside closes every level.
- A pointer hovering a submenu item opens it. A tap opens it on touch screens, where there is no
  hover.
- The menu uses the existing `.context-menu` styles. On the mobile layout it fits the viewport,
  and the touch rules keep the 44 px targets.

## Web deviations

- New Project always keeps the modified presets. Upstream asks Keep, Discard or Save
  (`check_and_keep_current_preset_changes`); the web takes the Keep answer and does not ask.
- The presets of an opened 3MF exist in the web only with that project's scene, so New Project
  drops them together with the project, the state a page reload also reaches. System and imported
  presets the user selected stay, with the sidebar edits.
- New Project's confirmation has Discard and Cancel only until the 3MF export slice adds Save.
- Until the 3MF export slice rebinds Ctrl+G, the Export G-code row shows Ctrl+G, because that key
  runs it. Upstream shows Ctrl+G on Export plate sliced file.

## Included

- The File bar button and menu with the rows above.
- The Import, Import Configs and Export G-code actions moved into it.
- New Window, New Project with its confirmation, Open Project and Ctrl+O.
- The title bar losing its Import and Import Configs buttons.

## Deferred

- Recent files and its per-viewer project store.
- Save Project, Save Project as, Export plate sliced file: the 3MF export slice.
- Export all plate sliced file and Export Generic 3MF: listed as deferred there.
- STL, DRC and toolpath OBJ export; ZIP import.
- Preset bundle dialog and export; cloud sync.
- Import adding to the current plate instead of replacing the model.

## Verification

A Playwright spec (`web/e2e/fileMenu.spec.ts`, desktop project, port 4241) covers:
- The File menu opens from the button and from the keyboard. Every row has the order, label,
  enabled state, accelerator and reason above.
- The spec writes the structure to `target/web-e2e/file-menu/menu.json`, including each
  submenu, and takes a screenshot of the open Export submenu.
- Escape closes a submenu and then the menu; a press outside closes it.
- Import 3MF/STL… through the menu (file chooser) loads the same scene as the
  `import-input` path: object count, plates and the status text.
- Open Project… loads a 3MF in the same way. Ctrl+O opens the same chooser.
- Import Configs… through the menu imports a preset file with the same result as
  `preset-import-input`.
- After a slice, File › Export › Export G-code… downloads bytes identical to the side button's
  download, with the same file name.
- New Project asks first. Cancel keeps the model; Discard empties the plate and keeps the
  selected presets.
- New Window opens a second page at the app's URL.
- A click on a disabled item neither closes the menu nor starts a download.

Existing specs that press the title bar's Import button (`mobile.spec.ts`) move to the File
menu in the same test commit. Specs that use the hidden inputs or the side buttons are
unchanged. The paired run of the whole web suite against the base shows no test going from
passing to failing.
