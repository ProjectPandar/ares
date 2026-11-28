import { useEffect, useRef, useState } from "react";
import { DialogHeader } from "./DialogHeader";
import { cloudHubUrl, type HubPrinter, listPrinters, signIn } from "../state/cloud";
import { type CalibrationMode, type PrintOptions, sendBlocker, sendPlate } from "../state/printHost";
import { useApp } from "../state/store";

function useModal(onClose: () => void) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => dialog.current!.showModal(), []);
  return { ref: dialog, onClose, close: () => dialog.current!.close() };
}

/**
 * The Cloud menu's sign-in (GUI_App::request_login, GUI_App.cpp:5363; ZUserLogin's WebView): the
 * Pandar Hub URL, then the Hub's own sign-in page in a popup until it hands back a ticket.
 */
export function CloudSignInDialog({ onClose, onSignedIn }: { onClose: () => void; onSignedIn: () => void }) {
  const { ref, close } = useModal(onClose);
  const [hubUrl, setHubUrl] = useState(cloudHubUrl);
  const [waiting, setWaiting] = useState<AbortController>();
  const [error, setError] = useState<string>();
  useEffect(() => () => waiting?.abort(), [waiting]);
  return (
    <dialog ref={ref} className="dialog" aria-labelledby="cloud-sign-in-title" onClose={onClose}>
      <DialogHeader id="cloud-sign-in-title" title="Sign in to Pandar Cloud" close={close} />
      <form
        className="dialog-form"
        onSubmit={(event) => {
          event.preventDefault();
          const controller = new AbortController();
          setWaiting(controller);
          setError(undefined);
          signIn(hubUrl, controller.signal).then(
            () => {
              close();
              onSignedIn();
            },
            (reason: Error) => {
              if (controller.signal.aborted) return;
              setWaiting(undefined);
              setError(reason.message);
            },
          );
        }}
      >
        <div className="dialog-body">
          <div className="stack">
            <label className="field">
              <span>Hub URL</span>
              <input
                type="url"
                required
                value={hubUrl}
                disabled={waiting !== undefined}
                placeholder="https://pandar.example.com"
                onChange={(event) => setHubUrl(event.target.value)}
              />
            </label>
            {waiting && (
              <p className="notice busy" role="status">
                <span className="spinner" aria-hidden="true" />
                Continue in the Pandar sign-in window…
              </p>
            )}
            {error && (
              <p className="notice error" role="alert">
                {error}
              </p>
            )}
          </div>
        </div>
        <footer className="dialog-footer">
          {waiting ? (
            <button type="button" className="button" onClick={() => setWaiting(undefined)}>
              Cancel
            </button>
          ) : (
            <button type="submit" className="button primary">
              Sign in
            </button>
          )}
        </footer>
      </form>
    </dialog>
  );
}

const MODES: [CalibrationMode, string][] = [
  [2, "Auto"],
  [1, "On"],
  [0, "Off"],
];

/** PrintHostSendDialog (PrintHostDialogs.cpp): picks a Hub printer and its print options, then uploads. */
export function PrintHostSendDialog({ onClose }: { onClose: () => void }) {
  const { ref, close } = useModal(onClose);
  const cloud = useApp((state) => state.cloud)!;
  const [printers, setPrinters] = useState<HubPrinter[]>();
  const [printerId, setPrinterId] = useState("");
  const [options, setOptions] = useState<PrintOptions>({
    useAms: false,
    bedLeveling: 2,
    flowCali: 2,
    offsetCali: 0,
    timelapse: false,
  });
  const [state, setState] = useState<{
    sending?: true;
    error?: string;
    sent?: true;
  }>({});
  useEffect(() => {
    listPrinters().then(
      (list) => {
        setPrinters(list);
        setPrinterId(list[0]?.id ?? "");
      },
      (error: Error) => setState({ error: error.message }),
    );
  }, []);
  const printer = printers?.find(({ id }) => id === printerId);
  const blocker = printer && sendBlocker(printer);
  const bambu = printer?.printer_family === "bambu_lab";
  const mode = (key: "bedLeveling" | "flowCali" | "offsetCali", label: string) => (
    <label className="field">
      <span>{label}</span>
      <select
        value={options[key]}
        onChange={(event) => setOptions({ ...options, [key]: Number(event.target.value) as CalibrationMode })}
      >
        {MODES.map(([value, text]) => (
          <option key={value} value={value}>
            {text}
          </option>
        ))}
      </select>
    </label>
  );
  const flag = (key: "useAms" | "timelapse", label: string) => (
    <label className="check">
      <input
        type="checkbox"
        checked={options[key]}
        onChange={(event) => setOptions({ ...options, [key]: event.target.checked })}
      />
      <span>{label}</span>
    </label>
  );
  return (
    <dialog ref={ref} className="dialog sheet" aria-labelledby="send-title" onClose={onClose}>
      <DialogHeader id="send-title" title="Send to Printer" close={close} />
      <form
        className="dialog-form"
        onSubmit={(event) => {
          event.preventDefault();
          setState({ sending: true });
          sendPlate(printer!, options).then(
            () => setState({ sent: true }),
            (error: Error) => setState({ error: error.message }),
          );
        }}
      >
        <div className="dialog-body">
          <div className="stack">
            <p className="muted">
              {cloud.tenantName} · {cloud.hubUrl}
            </p>
            <label className="field">
              <span>Printer</span>
              <select
                value={printerId}
                disabled={!printers?.length}
                onChange={(event) => setPrinterId(event.target.value)}
              >
                {printers?.length === 0 && <option value="">No printers</option>}
                {printers?.map(({ id, name, status }) => (
                  <option key={id} value={id}>
                    {name} ({status})
                  </option>
                ))}
              </select>
            </label>
            {bambu && (
              <section className="frame" aria-labelledby="print-options-title">
                <header className="frame-head">
                  <h2 id="print-options-title">Print options</h2>
                </header>
                <div className="frame-content">
                  <div className="grid-2">
                    {mode("bedLeveling", "Bed Leveling")}
                    {mode("flowCali", "Flow Dynamics Calibration")}
                    {mode("offsetCali", "Nozzle Offset Calibration")}
                  </div>
                  {flag("useAms", "Use AMS")}
                  {flag("timelapse", "Timelapse")}
                </div>
              </section>
            )}
            {(blocker ?? state.error) && (
              <p className="notice error" role="alert">
                {blocker ?? state.error}
              </p>
            )}
            {state.sent && (
              <p className="notice success" role="status">
                Sent to {printer!.name}.
              </p>
            )}
          </div>
        </div>
        <footer className="dialog-footer">
          <button
            type="submit"
            className="button primary"
            disabled={!printer || blocker !== undefined || state.sending || state.sent}
          >
            {state.sending ? "Sending…" : "Send"}
          </button>
        </footer>
      </form>
    </dialog>
  );
}
