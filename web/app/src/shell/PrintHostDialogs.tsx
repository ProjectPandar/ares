import { useEffect, useRef, useState } from "react";
import {
  type CalibrationMode,
  type HubPrinter,
  listPrinters,
  type PrintHost,
  type PrintOptions,
  printHost,
  sendBlocker,
  sendPlate,
  setPrintHost,
} from "../state/printHost";

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

/** PhysicalPrinterDialog (PhysicalPrinterDialog.cpp) for the `Pandar` host type. */
export function PhysicalPrinterDialog({ onClose, onSaved }: { onClose: () => void; onSaved: () => void }) {
  const { ref, close } = useModal(onClose);
  const [host, setHost] = useState<PrintHost>(printHost() ?? { hubUrl: "", tenantId: "", token: "" });
  const field = (key: keyof PrintHost, label: string, type = "text") => (
    <label className="print-host-field">
      {label}
      <input
        type={type}
        required
        value={host[key]}
        autoComplete="off"
        onChange={(event) => setHost({ ...host, [key]: event.target.value })}
      />
    </label>
  );
  return (
    <dialog ref={ref} className="print-host-dialog" aria-labelledby="physical-printer-title" onClose={onClose}>
      <Header id="physical-printer-title" title="Physical Printer" close={close} />
      <form
        className="print-host-body"
        onSubmit={(event) => {
          event.preventDefault();
          setPrintHost(host);
          close();
          onSaved();
        }}
      >
        <p className="print-host-type">Host Type: Pandar</p>
        {field("hubUrl", "Hub URL", "url")}
        {field("tenantId", "Tenant ID")}
        {field("token", "API Token", "password")}
        <footer className="vendor-footer">
          <button type="submit" className="side-button">
            OK
          </button>
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
export function PrintHostSendDialog({ onClose, onEditHost }: { onClose: () => void; onEditHost: () => void }) {
  const { ref, close } = useModal(onClose);
  const host = printHost()!;
  const [printers, setPrinters] = useState<HubPrinter[]>();
  const [printerId, setPrinterId] = useState("");
  const [options, setOptions] = useState<PrintOptions>({
    useAms: false,
    bedLeveling: 2,
    flowCali: 2,
    offsetCali: 0,
    timelapse: false,
  });
  const [state, setState] = useState<{ sending?: true; error?: string; sent?: true }>({});
  useEffect(() => {
    listPrinters(host).then(
      (list) => {
        setPrinters(list);
        setPrinterId(list[0]?.id ?? "");
      },
      (error: Error) => setState({ error: error.message }),
    );
  }, [host]);
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
    <dialog ref={ref} className="print-host-dialog" aria-labelledby="send-title" onClose={onClose}>
      <Header id="send-title" title="Send to Printer" close={close} />
      <form
        className="print-host-body"
        onSubmit={(event) => {
          event.preventDefault();
          setState({ sending: true });
          sendPlate(host, printer!, options).then(
            () => setState({ sent: true }),
            (error: Error) => setState({ error: error.message }),
          );
        }}
      >
        <p className="print-host-type">
          {host.hubUrl}{" "}
          <button type="button" className="link-button" onClick={onEditHost}>
            Edit
          </button>
        </p>
        <label className="print-host-field">
          Printer
          <select value={printerId} disabled={!printers?.length} onChange={(event) => setPrinterId(event.target.value)}>
            {printers?.length === 0 && <option value="">No printers</option>}
            {printers?.map(({ id, name, status }) => (
              <option key={id} value={id}>
                {name} ({status})
              </option>
            ))}
          </select>
        </label>
        {bambu && (
          <>
            {mode("bedLeveling", "Bed Leveling")}
            {mode("flowCali", "Flow Dynamics Calibration")}
            {mode("offsetCali", "Nozzle Offset Calibration")}
            {flag("useAms", "Use AMS")}
            {flag("timelapse", "Timelapse")}
          </>
        )}
        {(blocker ?? state.error) && (
          <p className="preset-error" role="alert">
            {blocker ?? state.error}
          </p>
        )}
        {state.sent && <p role="status">Sent to {printer!.name}.</p>}
        <footer className="vendor-footer">
          <button
            type="submit"
            className="side-button"
            disabled={!printer || blocker !== undefined || state.sending || state.sent}
          >
            {state.sending ? "Sending…" : "Send"}
          </button>
        </footer>
      </form>
    </dialog>
  );
}
