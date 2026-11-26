/**
 * The Devices tab's printers (DeviceManager's machine list, DeviceManager.cpp): the tenant's Hub
 * printers, then each `printer_snapshot` of the Hub's printer-event socket in their place.
 */
import { useEffect, useState } from "react";
import { type HubPrinter, listPrinters, tenantRequest, tenantUrl } from "./cloud";

/** The wait before reconnecting a closed event socket. */
const RECONNECT_MS = 5000;

type PrinterEvent = { type: "printer_snapshot"; printer: HubPrinter } | { type: "job_progress" | "command_result" };

export interface Devices {
  printers?: HubPrinter[];
  error?: string;
}

/** The signed-in tenant's printers, live while mounted. */
export function useDevices(): Devices {
  const [devices, setDevices] = useState<Devices>({});
  useEffect(() => {
    let socket: WebSocket | undefined;
    let retry: number | undefined;
    let stopped = false;
    const connect = async () => {
      try {
        const printers = await listPrinters();
        const { ticket } = (await tenantRequest("/printer-events/tickets", { method: "POST" })) as { ticket: string };
        if (stopped) return;
        setDevices({ printers });
        const url = new URL(tenantUrl("/printer-events"));
        url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
        url.searchParams.set("ticket", ticket);
        socket = new WebSocket(url);
        socket.onmessage = ({ data }: MessageEvent<string>) => {
          const event = JSON.parse(data) as PrinterEvent;
          if (event.type !== "printer_snapshot") return;
          const { printer } = event;
          setDevices(({ printers: list = [] }) => ({
            printers: list.some(({ id }) => id === printer.id)
              ? list.map((item) => (item.id === printer.id ? printer : item))
              : [...list, printer],
          }));
        };
        socket.onclose = () => {
          if (!stopped) retry = window.setTimeout(() => void connect(), RECONNECT_MS);
        };
      } catch (error) {
        if (stopped) return;
        setDevices((current) => ({ ...current, error: (error as Error).message }));
        retry = window.setTimeout(() => void connect(), RECONNECT_MS);
      }
    };
    void connect();
    return () => {
      stopped = true;
      window.clearTimeout(retry);
      socket?.close();
    };
  }, []);
  return devices;
}
