import { useEffect, useRef, useState } from "react";
import { cloudHubUrl, type HubPrinter, listPrinters, signIn } from "../state/cloud";
import { type CalibrationMode, type PrintOptions, sendBlocker, sendPlate } from "../state/printHost";
import { useApp } from "../state/store";

function useModal(onClose: () => void) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => dialog.current!.showModal(), []);
  return { ref: dialog, onClose, close: () => dialog.current!.close() };
}

function Header({ id, title, close }: { id: string; title: string; close: () => void }) {
  return (
    <header className="shortcuts-header">
      <h2 id={id}>{title}</h2>
      <button className="bar-button" aria-label="Close" onClick={close}>
        {"✕"}
      </button>
    </header>
  );
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
    <dialog ref={ref} className="print-host-dialog" aria-labelledby="cloud-sign-in-title" onClose={onClose}>
      <Header id="cloud-sign-in-title" title="Sign in to Pandar Cloud" close={close} />
      <form
        className="print-host-body"
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
        <div className="dialog-body print-host-fields">
          <label className="print-host-field">
            Hub URL
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
            <p className="dialog-notice busy" role="status">
              <span className="spinner" aria-hidden="true" />
              Continue in the Pandar sign-in window…
            </p>
          )}
          {error && (
            <p className="dialog-notice error" role="alert">
              {error}
            </p>
          )}
        </div>
        <footer className="vendor-footer">
          {waiting ? (
            <button type="button" className="side-button" onClick={() => setWaiting(undefined)}>
              Cancel
            </button>
          ) : (
            <button type="submit" className="side-button dialog-primary">
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
    <label className="print-host-field">
      {label}
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
    <label className="vendor-item">
      <input
        type="checkbox"
        checked={options[key]}
        onChange={(event) => setOptions({ ...options, [key]: event.target.checked })}
      />
      {label}
    </label>
  );
  return (
    <dialog ref={ref} className="print-host-dialog send-dialog" aria-labelledby="send-title" onClose={onClose}>
      <Header id="send-title" title="Send to Printer" close={close} />
      <form
        className="print-host-body"
        onSubmit={(event) => {
          event.preventDefault();
          setState({ sending: true });
          sendPlate(printer!, options).then(
            () => setState({ sent: true }),
            (error: Error) => setState({ error: error.message }),
          );
        }}
      >
        <div className="dialog-body print-host-fields">
          <p className="print-host-type">
            {cloud.tenantName} · {cloud.hubUrl}
          </p>
          <label className="print-host-field">
            Printer
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
            <section className="print-host-options" aria-labelledby="print-options-title">
              <h3 id="print-options-title">Print options</h3>
              <div className="print-host-options-body">
                {mode("bedLeveling", "Bed Leveling")}
                {mode("flowCali", "Flow Dynamics Calibration")}
                {mode("offsetCali", "Nozzle Offset Calibration")}
                {flag("useAms", "Use AMS")}
                {flag("timelapse", "Timelapse")}
              </div>
            </section>
          )}
          {(blocker ?? state.error) && (
            <p className="dialog-notice error" role="alert">
              {blocker ?? state.error}
            </p>
          )}
          {state.sent && (
            <p className="dialog-notice success" role="status">
              Sent to {printer!.name}.
            </p>
          )}
        </div>
        <footer className="vendor-footer">
          <button
            type="submit"
            className="side-button dialog-primary"
            disabled={!printer || blocker !== undefined || state.sending || state.sent}
          >
            {state.sending ? "Sending…" : "Send"}
          </button>
        </footer>
      </form>
    </dialog>
  );
}
