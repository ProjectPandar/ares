# One multi-part dialog per STL import

Upstream baseline: OrcaSlicer v2.5.0 (40ecc66f7).

## Upstream source

`Plater::priv::load_files` asks about a batch of STL files with a modal dialog of its own
(`slic3r/GUI/Plater.cpp:10097-10102`): `RichMessageDialog` "An object with multiple parts was
detected", the "Auto-Drop" check box checked, then `ShowModal()`. Each load builds and shows a new
dialog. Its answer and check box apply to that load only, and nothing carries over to the next one.

## Ares destination

- `web/app/src/state/store.ts` `stlImport`: the batch awaiting its answer, with an id per import.
- `web/app/src/state/actions.ts` `importFiles`: each STL batch is a new import with a new id.
- `web/app/src/shell/Shell.tsx`: the dialog is keyed by that id, so every import mounts its own
  `MultipartDialog`, which opens itself (`showModal`). A dialog's close clears the batch
  only while that batch is still the one it was opened for.

## Included

A batch that arrives while the previous dialog is closing gets its own dialog. Escape closes the
`<dialog>` at once, but its `close` event comes later as a separate task. Today a drop in that gap
replaces the batch under the dialog that is still mounted, which does not open again, and the late
`close` event then clears the new batch: the drop is lost.

## Deferred

None. The dialog's text, check box and answers are unchanged.

## Verification

`web/e2e/multipartImport.spec.ts`: closes the dialog as Escape does and drops two STL files in the
same task, before the `close` event. The new dialog must open with Auto-Drop checked. The same
drop after the `close` event already works and stays covered by the existing batch replacement
case.
