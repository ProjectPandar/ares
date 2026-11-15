# Web: wasm panic message and engine recovery

## Problem

A Rust panic in `ares-wasm` aborts with a WebAssembly trap. The browser reports only
`RuntimeError: unreachable`, without the panic message or its location. The async exports run
their first poll in a `wasm-bindgen-futures` microtask, so the trap escapes the worker's request
`try`/`catch`: the request never settles, and only the main thread's `worker.onerror` fires. Later
requests enter the same instance after the trap, with a broken stack pointer and a held
`RefCell` borrow. They fail with misleading follow-on errors such as `memory access out of bounds`
or `recursive use of an object detected`.

## Upstream boundary

OrcaSlicer reports a fatal error in the background slicing process as a `HardCrash`.
`SlicingProcessCompletedEvent::format_error_message`
(`src/slic3r/GUI/BackgroundSlicingProcess.cpp:110-112`) formats it as:

```text
A fatal error occurred: "<what>"
Please save your project and restart the application.
```

The web engine reports a wasm trap with this text. `<what>` is the panic hook's message, which
holds the panic message and `file:line:column`. Upstream has no automatic recovery: the user
restarts the application. The recovery dialog below is Ares UI with no upstream counterpart.

## Decisions

1. **Panic hook.** `ares-wasm` gets a `#[wasm_bindgen(start)]` function that installs
   `std::panic::set_hook`. The hook formats the `PanicHookInfo` `Display` and passes it
   synchronously, before the abort, to the JS import `aresWasmPanic`, a function the worker
   defines on its global scope before instantiating the module. Nothing calls into wasm after the
   trap. An allocation failure aborts without a panic and so without the hook; its report carries
   the trap text instead.
2. **Crash detection in the worker.** The worker treats a `WebAssembly.RuntimeError` as a crash,
   whether it reaches the request's `catch` or the worker's global `error` event (the microtask
   case). The worker cancels the global event so the generic error is not also relayed, then posts
   one crash message with the HardCrash text: the hook's message, or the trap text when no hook
   message arrived.
3. **Failed engine.** After a crash the engine is failed for good. The worker answers every later
   request with the same HardCrash text without entering wasm. The client (`AresSlicer`) rejects
   every open request and every later call with that text, and reports the crash once through
   its `onCrash` option.
4. **Recovery (Ares UI).** The app shows the HardCrash text in the status bar and in a modal
   dialog with one action, which reloads the page. Before reloading, the action stores the
   current file's original bytes and name and the preset selection (the selected printer, or the
   project's presets) as one IndexedDB record. The new page gets a new worker and wasm instance.
   Startup restores the app config selection as usual. The file is then opened again from the
   stored bytes, and a stored printer is selected again, which brings back its process,
   filaments and bed type from the app config. The record is deleted as soon as the recovery
   reads it, whether the reload succeeds or fails: user files are never kept in IndexedDB. The
   dialog states that edits made on the canvas after loading, and the undo history, are lost.
5. **Test trigger.** A non-default `panic-probe` cargo feature of `ares-wasm` exports
   `panicProbe()`. With the feature on, the plater session's `sliceWithProfilesProcessed` export
   calls it first. The shipped wasm is built without the feature and contains neither.
   `prepare:e2e` builds a second wasm with the feature into `target/web-e2e/panic-probe/`. The
   spec serves it in place of the app's wasm for the first fetch only, so the first slice panics
   on the real in-flight path, and the reloaded page loads the shipped wasm. The spec asserts that
   the shipped wasm, the package `dist` and the app assets do not contain `panicProbe`.

## Verification

`web/e2e/wasmPanic.spec.ts`:

- Slicing with the probe wasm shows the HardCrash text in the status bar and in the dialog,
  with the probe's `file:line` read from its source.
- A second request after the crash fails with the same text. No `unreachable`,
  `memory access out of bounds` or `recursive use` text appears in the UI or the console.
- The dialog's action reloads the page with the same file and preset selection, slices
  normally, and leaves no recovery record in IndexedDB.
