import type { HubPrinter } from "../state/cloud";
import { useDevices } from "../state/devices";

const temperature = (current?: string | null, target?: string | null) =>
  current == null ? "—" : target && Number(target) > 0 ? `${current} / ${target} °C` : `${current} °C`;

const remaining = (minutes: number) =>
  minutes >= 60 ? `${Math.floor(minutes / 60)} h ${minutes % 60} min` : `${minutes} min`;

/** StatusPanel's temperatures and print progress (StatusPanel.cpp) for one Hub printer. */
function DeviceCard({ printer }: { printer: HubPrinter }) {
  const { print } = printer;
  const active = print?.gcode_state && !["IDLE", "FINISH", "FAILED"].includes(print.gcode_state);
  const rows: [string, string][] = [
    ...(printer.nozzle_temperatures ?? []).map(
      ({ label, current_celsius, target_celsius }, index): [string, string] => [
        label ?? `Nozzle ${index + 1}`,
        temperature(current_celsius, target_celsius),
      ],
    ),
    ["Bed", temperature(printer.bed_temperature_celsius, printer.bed_target_temperature_celsius)],
  ];
  if (printer.chamber_temperature_celsius != null)
    rows.push([
      "Chamber",
      temperature(printer.chamber_temperature_celsius, printer.chamber_target_temperature_celsius),
    ]);
  return (
    <article className="device-card" aria-label={printer.name}>
      <header className="device-card-header">
        <h3>{printer.name}</h3>
        <span className="device-status" data-status={printer.status}>
          {printer.status}
        </span>
      </header>
      {printer.model && <p className="device-model">{printer.model}</p>}
      <dl className="device-values">
        {rows.map(([name, value]) => (
          <div key={name}>
            <dt>{name}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
      {active && (
        <div className="device-print">
          <p>
            {print.subtask_name ?? print.gcode_state}
            {print.current_layer != null && print.total_layers != null && (
              <span>
                {" "}
                · Layer {print.current_layer}/{print.total_layers}
              </span>
            )}
            {print.remaining_time_minutes != null && <span> · {remaining(print.remaining_time_minutes)} left</span>}
          </p>
          <progress max={100} value={print.progress_percent ?? 0} aria-label={`${printer.name} progress`} />
        </div>
      )}
    </article>
  );
}

/** MainFrame's Device tab (MonitorPanel, MainFrame.cpp:1606-1608): the signed-in tenant's printers. */
export function DevicesPage() {
  const { printers, error } = useDevices();
  return (
    <section className="devices-page" aria-label="Devices">
      {error && (
        <p className="preset-error" role="alert">
          {error}
        </p>
      )}
      {printers?.length === 0 && <p className="devices-empty">No printers in this Pandar tenant.</p>}
      {!printers && !error && <p className="devices-empty">Loading printers…</p>}
      <div className="devices-grid">
        {printers?.map((printer) => (
          <DeviceCard key={printer.id} printer={printer} />
        ))}
      </div>
    </section>
  );
}
