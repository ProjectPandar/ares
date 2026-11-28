# Design

## Source of truth

- Status: **Active** for the dialog set and the token layer; **Draft** for the rest of the workspace,
  whose artboards have not yet been audited against the app.
- Last refreshed: 2026-11-28.
- Primary product surfaces: the Ares web app (`web/app`) — Prepare (3D view and sidebar), Preview,
  Devices — and its dialog set: Help, File, object, preset, Cloud, Send to Printer, the first-run
  setup wizard, the arrange popup and engine recovery.
- Evidence reviewed:
  - The Design canvas workspace `design/` (local only: `/design/` is git-ignored, see
    `design/README.md`): `ares-slicer-dialogs/` (25 artboards, `assets/dialogs.css`,
    `assets/dialogs.js`, `design.json`) and `ares-slicer-ui/` (16 artboards,
    `assets/coss-workspace.css`, comments 1–6, all resolved).
  - App styles: `web/app/src/styles/tokens.css`, `dialogs.css`, `dialogPanels.css`, `shell.css`,
    `sidebar.css`, `viewport.css`, `mobile.css`, `touch.css`, `compact.css`.
  - Dialog components: `shell/DialogHeader.tsx`, `AboutDialog`, `ShortcutsDialog`,
    `EngineCrashDialog`, `MultipartDialog`, `PrintHostDialogs`, the New Project dialog of `FileMenu`;
    `sidebar/CopiesDialog`, `ExtruderGroups` (Switch diameter), `FilamentSlotMenu` (Merge filament),
    `SettingsDialog` + `SettingsPages`, `FilamentDialog`; `viewport/ArrangeMenu`; `wizard/SetupWizard`,
    `ModelPage`, `FilamentPage`.
  - Tests that encode the constraints: `web/e2e/dialogDesign.spec.ts` (this contract's parity
    check), `touch.ts` (44 px targets), `a11y.ts` (16 px fields), `settings.spec.ts` (axe),
    `about.spec.ts`, `wizard*.spec.ts`, `pandar*.spec.ts`.
  - `web/README.md`, `AGENTS.md`.

## Brand

- Personality: compact, calm, technical — "the Compact coss UI style" the dialog artboards declare.
- Trust signals: the OrcaSlicer-compatible engine and its wording; visible state (engine dot,
  status bar, disabled actions with a reason); attribution to Slic3r, PrusaSlicer, Bambu Studio and
  OrcaSlicer in About.
- Avoid: padding-heavy rows (user comments #2, #3 and #6 on `ares-slicer-ui`: "padding 太多了"),
  text toolbars where a spatial control fits (comment #1: the Bambu-style view cube), and browser
  default dialog chrome — before this refresh the unstyled dialogs showed the UA's 3 px border.

## Product goals

- Goals: slice in the browser with OrcaSlicer's engine and vocabulary, so an OrcaSlicer user finds
  every dialog where they expect it; run on WASM, Windows, macOS and Linux; keep slicing logic in
  `ares-core` behind an API, with the UI calling that API.
- Non-goals: an Ares-designed slicing pipeline; chrome that departs from upstream wording.
- Success signals: byte-identical G-code in the e2e parity specs; touch and axe checks pass; the
  dialog parity check of this document passes where `design/` exists.

## Personas and jobs

- Primary personas: 3D-printing owners who know OrcaSlicer or Bambu Studio; operators of a Pandar
  Hub with Bambu and Klipper printers.
- User jobs: set up printers and filaments on first run; arrange objects; slice; inspect; send a
  plate to a printer; edit presets; recover from an engine crash.
- Key contexts of use: a desktop with a mouse; tablets and phones with touch (`*-mobile` artboards,
  the `iphone`, `pixel` and `ipad` e2e projects).

## Information architecture

- Primary navigation: title bar (brand, File, Cloud, title, Help, theme), tab bar (Prepare / Preview
  tabs; Slice and Output split actions), status bar.
- Core screens: Prepare = 3D view with overlays (view cube, toolbar, arrange popup, slicing result)
  + sidebar (printer, filament, objects, process); Preview; Devices when signed in.
- Dialog inventory (artboard → component → trigger):

  | Artboards | Component | Opens from |
  | --- | --- | --- |
  | `cloud-sign-in`, `cloud-waiting`, `cloud-error` | `CloudSignInDialog` | Cloud menu → Sign in… |
  | `new-project` | `NewProjectDialog` (`shell/FileMenu.tsx`) | File → New Project with a project open |
  | `multipart` | `MultipartDialog` | importing two or more STL files |
  | `copies` | `CopiesDialog` | object menu → Set number of instances… |
  | `arrange` | `ArrangePopup` | the toolbar's Arrange |
  | `switch-diameter` | `SwitchDiameter` (`sidebar/ExtruderGroups.tsx`) | a second nozzle of another diameter |
  | `merge-filament` | merge confirmation of `FilamentSlotMenu` | Merge with a mixed filament that uses this slot |
  | `printer-settings`, `printer-settings-mobile` | `SettingsDialog` (printer) | Edit printer settings |
  | `filament-settings` | `SettingsDialog` (filament) | Edit filament N settings |
  | `filament-library`, `filament-library-mobile` | `FilamentDialog` | Set filaments to use |
  | `wizard-printers`, `wizard-filaments`, `wizard-filament-error` | `SetupWizard` (first run) | a first run without printers |
  | `manage-printers` | `SetupWizard` (models page) | printer combo → Select/Remove printers |
  | `send-to-printer`, `send-blocked`, `send-sent`, `send-mobile` | `PrintHostSendDialog` | Print plate, signed in |
  | `shortcuts`, `about` | `ShortcutsDialog`, `AboutDialog` | Help menu |
  | `engine-crash` | `EngineCrashDialog` | a worker crash |

## Design principles

1. **One dialog anatomy.** Every dialog and popup is `.dialog` (or the `.popup`) with a
   `.dialog-header` (`.dialog-heading` > `h1`, optional `p`), a scrolling `.dialog-body` and a
   `.dialog-footer`. No dialog styles anything outside this vocabulary.
2. **Upstream wording, Ares chrome.** Labels, messages and button texts keep the OrcaSlicer text the
   components cite; the artboards' paraphrases are placeholders (Decision D2).
3. **Compact by default.** 34 px controls, 30 px checkbox rows, 17/13 px type, 20 px padding; capped
   heights with inner scrolling instead of growing pages.
4. **Tokens with light/dark pairs.** A colour that exists in one theme exists in the other; the
   dialog tokens are aliases of the workspace tokens (table below).
5. **Touch targets hold.** Compact rows keep 44 px targets under `(pointer: coarse)`.

Tradeoffs: density against touch targets (solved by media queries, not by loosening either);
artboard pixel parity against real content (look compared, content reported).

## Visual language

- Color — dialog tokens (`web/app/src/styles/dialogs.css`) and the workspace token they pair with:

  | Token | Light | Dark | Workspace token |
  | --- | --- | --- | --- |
  | `--panel` | `#fff` | `#1c1e22` | same |
  | `--dialog-muted` | `#f2f3f5` | `#25282d` | between `--panel-raised` and `--field` |
  | `--field` | `#fff` | `#25282d` | same |
  | `--border` | `#e0e2e6` | `#373b42` | same |
  | `--text` | `#181b20` | `#f2f3f5` | same |
  | `--text-dim` | `#5d646e` | `#aaafb8` | same |
  | accent fill (buttons, checked) | `--accent-fill` `#08766c` | `#87e4d3` | same |
  | `--accent-text` | `#fff` | `#101f1d` | same (D8) |
  | `--accent-soft` | `#e8f6f3` | `#223632` | same |
  | `--dialog-danger-text` / `-fill` / `-soft` | `#ae2d35` / `#b82f38` / `#fff1f1` | `#ffa3aa` / `#ae3945` / `#3d252b` | dialog only |
  | `--dialog-warning` / `-soft` | `#855211` / `#fff5e3` | `#ffd38c` / `#3b3021` | dialog only |
  | `--dialog-scrim` | `#171c264e` | `#08090cb0` | dialog only |

- Typography: dialog text `13px/1.46 Inter, "Segoe UI", system-ui, sans-serif` (the artboard body
  font; Inter is not bundled, both fall back alike). Title `h1` 17 px / 1.3, weight 650, tracking
  −0.015 em; description and help text `--text-dim`; labels and buttons weight 550; frame titles,
  tabs, summaries and steps 12 px; catalogue action buttons and `kbd` 11 px; About name 16 px,
  About mark 23 px / 700; numeric fields `tabular-nums`.
- Spacing: `--dialog-gap` 12 px, `--dialog-padding` 20 px (15 px on phones), header
  `17 / padding / 10`, body `3 / padding / 16`, footer `10 / padding`, footer button gap 8 px, field
  gap 5 px, check row 30 px, `--dialog-control-height` 34 px (44 px on phones and coarse pointers).
- Shape: dialog radius 16 px (sheets: top corners only); controls, fields and notices 8 px; frames
  11 px; vendor nav 10 px; segmented tabs 9 px; `kbd` 5 px.
- Elevation: dialog shadow `0 1px 2px #0002, 0 16px 48px #0a0d1533`; scrim per theme; primary
  button highlight `inset 0 1px #ffffff24`.
- Motion: the busy spinner turns 1 s per revolution and stops under `prefers-reduced-motion`
  (the app-wide rule); nothing else animates.
- Iconography: the close button is the `×` glyph at 19 px in a 32 px square (44 px on phones); the
  About mark is the letter A on the accent; toolbar icons stay the OrcaSlicer set.

## Components

- Existing components to reuse: `DialogHeader` (title, description, close), the anatomy classes
  above, `ModeSwitch` inside the settings header, `SettingsPages` inside the settings body.
- Sizes: `.dialog` 460 px; `.wide` 790 px (catalog, wizard); `.settings` 700 px; `.popup` 336 px;
  every one `min(--width, 100% − 36px)` wide and at most `100% − 36px` tall.
- Footer variants: primary at the right (Cancel · Discard, No · Yes, Cancel · OK); several choices
  as plain buttons (Switch diameter); a summary at the left (catalog, wizard); none where the
  artboard has none (About, settings).
- Panels (`dialogPanels.css`): `.frame` with `.frame-head` and `.frame-content`; `.catalog` of three
  frames (156 / 164 / rest) with `.search` and `.check` rows carrying a `.check-detail`;
  `.wizard` = `.vendor-nav` + a stack of a `.search` and vendor frames of `.models` (three
  `.model` cards per row, two on phones); settings pages = segmented `.process-tabs` over framed
  two-column rows (`minmax(0, 1fr) min(44%, 210px)`); `.shortcuts-list` rows (description, keys);
  `.about-identity`; the popup's `.spacing-row` (range + number).
- States: `.notice` neutral, `.error`, `.warning`, `.success`, `.busy` (with `.spinner`); `.button`
  default, `.primary`, `.danger`, `.close`, `:disabled` at 0.56 opacity; `.model:has(input:checked)`
  on the accent soft; `.dialog.sheet` as a bottom sheet on phones (Send, filament selection,
  settings).
- Token/component ownership: tokens in `tokens.css` and the `:root` block of `dialogs.css`; the
  anatomy in `dialogs.css`; panels in `dialogPanels.css`. Both are the artboard stylesheet
  `design/ares-slicer-dialogs/assets/dialogs.css` with its tokens renamed; change them together.
- Nesting: the wizard's Error is a second `<dialog>` in the top layer over the first; it declares
  the default width itself (a `.wide` parent's `--width` would otherwise be inherited) and stops its
  `close` and `cancel` events, which React bubbles to the dialog under it.

## Accessibility

- Target standard: WCAG 2.1 AA plus 2.5.5 (44 px) on touch; axe runs on the settings dialogs.
- Keyboard/focus behavior: `<dialog>` modality traps focus and Escape closes, except the forced
  dialogs (`data-required` in the artboards: first run, engine crash). The artboard prototype
  focuses the heading; the app keeps focusing its first control, which its tests pin (D5).
- Screen-reader semantics: `role="alertdialog"` on Discard project, Switch diameter, Merge filament,
  Slicing engine failed and the wizard's Error (D6); `aria-labelledby` the `h1`; `aria-describedby`
  the multipart question; `role="status"` and `role="alert"` on notices.
- Contrast/readability (WCAG ratios, computed): dim text on the panel 6.0:1 light and 7.6:1 dark;
  accent text on the accent fill 5.5:1 and 11.4:1; white on the danger fill 6.0:1 and 6.1:1; danger and
  warning text on their soft backgrounds 5.9–9.2:1; text on the muted footer and frame heads above
  13:1.
- Reduced motion: the app-wide `prefers-reduced-motion` rule stops the spinner.

## Responsive behavior

- Supported breakpoints/devices: ≥ 641 px desktop layouts; ≤ 640 px phone layouts; tablets at
  641–1024 px keep desktop layouts; `(pointer: coarse)` adds touch targets at any width.
- Layout adaptations (≤ 640 px): padding 15 px, controls 44 px, dialogs `100% − 20px` wide,
  `.dialog.sheet` anchored to the bottom, footer buttons stretch, `.grid-2`, `.catalog` and
  `.wizard` single column, vendor nav scrolls sideways, two model cards per row, close button 44 px.
- Touch/hover differences: coarse pointers get 44 px buttons, checks, tabs, catalogue actions and
  fields and 16 px field text (no iOS zoom); hover colours only apply under `hover: hover`.

## Interaction states

- Loading: a `.notice.busy` with the spinner (Cloud sign-in waiting); "Loading printers…" and
  "Loading filaments…" bodies in the wizard and catalogue.
- Empty: "No items" rows in a filtered catalogue; the wizard's Next disabled until a model is checked.
- Error: `.notice.error` — Hub errors, the send blocker ("Slice the plate first.", multi-plate to a
  Bambu printer), the wizard's nested Error dialog, the engine crash dialog.
- Success: `.notice.success` "Sent to <printer>." with Send disabled.
- Disabled: `.button:disabled` at 0.56; a disabled field at 0.65.

## Content voice

- Tone: plain, imperative, the upstream strings ("An object with multiple parts was detected").
- Terminology: OrcaSlicer's — printer, filament, process, plate, nozzle; "Pandar Cloud" for the Hub.
- Microcopy rules: keep upstream wording where the components cite it (D2); descriptions are real
  data (the selected preset under a settings title, the tenant and Hub above the printer list).

## Implementation constraints

- Framework/styling system: React 19 + Vite, plain CSS with custom properties and native nesting;
  no component library. Dialog styles live in two files scoped under `.dialog, .popup`.
- Design-token constraints: the dialog tokens alias workspace tokens; never hard-code a colour in a
  component. `--accent-text` is `#fff` in light (D8).
- Performance constraints: the bundle budget of `web/e2e/bundle.spec.ts`; dialogs add no library.
- Compatibility constraints: `color-mix()`, `:has()`, CSS nesting and `dvh` units (evergreen
  browsers, the Tier-1 set).
- Test/screenshot expectations: `web/e2e/dialogDesign.spec.ts` renders every artboard of
  `design/ares-slicer-dialogs` (25) and the app in the same state, theme and viewport, then compares the
  computed styles of each part of the anatomy (must be equal) and the dialog's pixels (reported).
  Artifacts: `target/web-e2e/dialog-design/<artboard>/` and the gallery `index.html`. The spec is
  skipped where `design/` is absent (CI). Content differences are listed in each result's `notes`.

## Decisions

- D1: the app uses the artboards' class names, so the parity check reads the same selectors on both
  sides and the stylesheet can be diffed against the artboard's.
- D2: where the artboards paraphrase upstream text (Switch diameter's message and buttons, "Number
  of copies", shortcut wording, About credits, the engine crash message), the app keeps its cited
  upstream wording and real data; settings group titles keep the upstream icons before the title.
- D3: long lists scroll in place — a catalogue frame's row list, the wizard's vendor nav and its
  vendor list are capped at `100dvh` less the header, body padding and footer heights on ≥ 641 px —
  so the filters, the search and the vendor nav stay in view, as OrcaSlicer's own columns do.
- D4: the arrange popup keeps its app anchoring (centred under the toolbar, `top: 58px`, 70 px on
  touch); the artboard's fixed coordinates belong to its mock background.
- D5: focus order is unchanged (first control), as the existing e2e specs assert.
- D6: forced-choice dialogs carry `role="alertdialog"`, as the artboards mark them.
- D7: under `(pointer: coarse)` every dialog control reaches 44 px, overriding the artboard's 38 px
  phone tabs and catalogue actions and 36 px fields, which apply to fine-pointer windows only.
- D8: `--accent-text` in light is `#fff`, the artboard value (was `#fefefe`).

## Open questions

- [ ] Inter is listed first but not bundled; should the app ship it (weight, licence) so Windows and
  macOS render as the artboards do on this Linux host? / frontend / typography parity.
- [ ] On phones the wizard stays a centred dialog, not a sheet (no `*-mobile` wizard artboard);
  confirm / design / reachability on small screens.
- [ ] Should a mixed-filament project fixture join `tests/parity` so the Merge filament confirmation
  has a stable e2e of its own? / slicing / coverage.
- [ ] The `ares-slicer-ui` artboards (workspace, sidebar, preview) have not been audited against the
  app with the same parity check / design / next refresh.
