# Web: Help menu and About dialog

## Problem

The web app has no way to show which Ares release a user is running. OrcaSlicer's top bar
has a Help menu that ends with "About OrcaSlicer", which opens a dialog with the version.
The web top bar has only a `?` button for the keyboard shortcut list.

## Upstream reference

OrcaSlicer v2.5.0 (`40ecc66f7`), GUI. This is a layout reference only; no `libslic3r`
behaviour is involved.

- `src/slic3r/GUI/MainFrame.cpp:2903-2966`, `generate_help_menu`: the Help menu starts with
  "Keyboard Shortcuts" (`:2907-2909`) and ends with "About <app name>" (`:2958-2963`),
  which calls `Slic3r::GUI::about()`. `:3683` puts the Help menu in the top bar's drop-down
  (`AddDropDownSubMenu(helpMenu, _L("Help"))`).
- `src/slic3r/GUI/AboutDialog.cpp:230-360`, `AboutDialog`:
  - the title "About <app>" (`:231`);
  - the logo (`:252-257`);
  - the version string, large, grey, right-aligned (`:259-279`);
  - the attribution text naming Slic3r, PrusaSlicer, Bambu Studio and OrcaSlicer
    (`:289-292`);
  - the copyright line and the project link (`:335-357`).
- `src/slic3r/GUI/AboutDialog.cpp:145-175`, `CopyrightsDialog::get_html_text`: the licence,
  "GNU Affero General Public License, version 3", linked to
  `https://www.gnu.org/licenses/agpl-3.0.html`.

## Version sources

- The Ares version is the `version` field of `web/app/package.json`. Each batch release
  bumps it together with `[workspace.package] version` in `Cargo.toml`. Vite reads the
  field at build time and injects it through `define`. No other file holds the value.
- The OrcaSlicer version Ares tracks is `ares_core::ORCA_SLICER_COMPATIBILITY_VERSION`.
  `ares-wasm` exports it as `orcaSlicerVersion()`, and the client package exposes it through
  the worker.

## Destination

- `web/app/vite.config.ts`: `define` injects `ARES_VERSION` from `package.json`.
- `web/app/src/shell/TopBar.tsx`: a Help menu button in the title bar, with
  "Keyboard Shortcuts" and "About Ares".
- `web/app/src/shell/AboutDialog.tsx`: the About dialog, a modal `<dialog>` like
  `ShortcutsDialog`.
- `crates/ares-wasm/src/lib.rs`, `web/packages/ares-wasm-js`: the `orcaSlicerVersion()`
  export and its client method.

## Included

- Help menu with "Keyboard Shortcuts", which opens the existing list, and "About Ares".
- The `?` key still opens the shortcut list.
- About dialog with:
  - the title "About Ares";
  - the version, large;
  - one line naming Ares as a Rust port of the OrcaSlicer slicing core and the OrcaSlicer
    version it tracks;
  - the attribution text;
  - the licence, AGPL-3.0, with its link;
  - closing on Esc and on the close button.
- Light and dark themes, and phone width.

## Deferred

- Check for Updates and the other Help menu entries (`MainFrame.cpp:2910-2956`).
- The `CopyrightsDialog` list of third-party components (`AboutDialog.cpp:62-145`).
- A logo. Ares has none, so the dialog uses text only, and Orca's brand assets are not
  copied.
- A build commit label (`AboutDialog.cpp:266`).

## Failure modes and how each is ruled out

1. **The shown version is not the released one.** The test reads the expected value from
   `web/app/package.json` at run time, not from a literal.
2. **The OrcaSlicer version drifts from the core's.** The dialog reads it from the WASM
   export of the core constant, and the test compares it with the constant's value in
   `crates/ares-core/src/generation.rs`.
3. **The menu replaces the shortcut list's entry points.** The test opens Keyboard Shortcuts
   from the menu. The existing shortcuts specs keep opening it with `?`.
4. **The dialog does not close.** The test closes it with Esc, and once with the close
   button.
5. **The dialog overflows a phone screen.** The test opens it at 390 px wide and checks that
   it fits the viewport.

## Tests

`web/e2e/about.spec.ts`, in the desktop project, opens the app with the stored first-run
config and checks:
- the Help menu lists exactly "Keyboard Shortcuts" and "About Ares", and closes after a
  pick;
- Keyboard Shortcuts opens the shortcut list, and Esc closes it;
- About Ares opens a dialog titled "About Ares", whose version is the `version` of
  `web/app/package.json`, read when the test runs;
- the dialog names the OrcaSlicer version of `ORCA_SLICER_COMPATIBILITY_VERSION` in
  `crates/ares-core/src/generation.rs`, read when the test runs, the four attributed
  projects, and the AGPL-3.0 link `https://www.gnu.org/licenses/agpl-3.0.html`;
- Esc closes the dialog, and so does its close button;
- the dialog's background differs between the dark and the light theme;
- at 390 px wide the dialog lies within the viewport, and the page does not scroll
  sideways;
- the page raises no errors.

It writes the two versions, the theme backgrounds and the phone-width dialog box to
`result.json`, with a screenshot of each state.

Neighbouring specs, run before and after the change:
- `shortcuts.spec.ts` and `plateDeleteAll.spec.ts`, which open the shortcut list with the
  `?` key;
- `mobile.spec.ts` in the Chromium phone project, which drives the top bar at phone width.
