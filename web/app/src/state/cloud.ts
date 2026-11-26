/**
 * The Cloud menu's Pandar Hub account: GUI_App::request_login and request_user_logout
 * (GUI_App.cpp:5363-5470) over the Hub's web login ticket. A popup signs in on `{hub}/web-sign-in`;
 * the Hub redirects it to `cloud-callback.html` with a one-use ticket, which the page broadcasts
 * back here (the app's `Cross-Origin-Opener-Policy` severs `window.opener`), and the ticket and
 * the PKCE verifier exchange for a `web:session` tenant token.
 */
import { saveConfig, storedConfig } from "./persistence";
import { useApp } from "./store";

/** The BroadcastChannel `cloud-callback.html` posts `{ ticket, state }` on. */
const CHANNEL = "ares.cloud";

export interface HubNozzleTemperature {
  label: string | null;
  current_celsius: string | null;
  target_celsius: string | null;
}

export interface HubPrint {
  gcode_state: string | null;
  subtask_name: string | null;
  progress_percent: number | null;
  remaining_time_minutes: number | null;
  current_layer: number | null;
  total_layers: number | null;
}

/** A Hub `Printer` (contracts/hub-client.openapi.json), as far as Ares reads it. */
export interface HubPrinter {
  id: string;
  name: string;
  model: string | null;
  printer_family: "bambu_lab" | "moonraker";
  status: string;
  nozzle_temperatures?: HubNozzleTemperature[];
  bed_temperature_celsius?: string | null;
  bed_target_temperature_celsius?: string | null;
  chamber_temperature_celsius?: string | null;
  chamber_target_temperature_celsius?: string | null;
  print?: HubPrint | null;
}

interface TicketExchange {
  token: string;
  expires_at: string;
  profile: { user_name: string; tenant_id: string; tenant_name: string };
}

/** The Hub URL of the last sign-in, for the next one. */
export const cloudHubUrl = () => storedConfig.cloud?.hubUrl ?? "";

const base64url = (bytes: Uint8Array) =>
  btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
const randomToken = (size: number) => base64url(crypto.getRandomValues(new Uint8Array(size)));

async function hubJson(response: Response): Promise<unknown> {
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { error?: string } | null;
    throw new Error(`Pandar Hub: ${body?.error ?? `HTTP ${response.status}`}`);
  }
  return response.json();
}

/** Signs in to `hubUrl` in a popup; resolves once signed in. Aborting `signal` stops waiting for the popup. */
export async function signIn(hubUrl: string, signal: AbortSignal) {
  const hub = hubUrl.replace(/\/+$/, "");
  const verifier = randomToken(32);
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(verifier));
  const state = randomToken(16);
  const signInUrl = new URL(`${hub}/web-sign-in`);
  signInUrl.search = new URLSearchParams({
    redirect_url: new URL("cloud-callback.html", location.origin + location.pathname).href,
    code_challenge: base64url(new Uint8Array(digest)),
    state,
  }).toString();
  if (!window.open(signInUrl, "ares-cloud-sign-in", "popup,width=520,height=720"))
    throw new Error("The browser blocked the sign-in window. Allow pop-ups for this site and try again.");
  // Listening starts in this task, before the popup can load its callback.
  const channel = new BroadcastChannel(CHANNEL);
  try {
    const ticket = new Promise<string>((resolve, reject) => {
      signal.addEventListener("abort", () => reject(signal.reason as Error));
      channel.onmessage = ({ data }: MessageEvent<{ ticket: string | null; state: string | null }>) => {
        if (data.state !== state) return;
        if (data.ticket) resolve(data.ticket);
        else reject(new Error("Pandar Hub sign-in returned no ticket."));
      };
    });
    const response = await fetch(`${hub}/api/v1/web/login-tickets/exchange`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ticket: await ticket, code_verifier: verifier }),
    });
    const { token, expires_at, profile } = (await hubJson(response)) as TicketExchange;
    const session = {
      token,
      expiresAt: expires_at,
      tenantId: profile.tenant_id,
      tenantName: profile.tenant_name,
      userName: profile.user_name,
    };
    saveConfig({ cloud: { hubUrl: hub, session } });
    useApp.setState({ cloud: { hubUrl: hub, ...session } });
  } finally {
    channel.close();
  }
}

/** Forgets the session, keeping the Hub URL for the next sign-in. */
export function signOut() {
  const { cloud } = useApp.getState();
  if (!cloud) return;
  saveConfig({ cloud: { hubUrl: cloud.hubUrl } });
  useApp.setState({ cloud: null });
}

/** The signed-in tenant's URL for `path` (`/printers`, ...). */
export function tenantUrl(path: string) {
  const { hubUrl, tenantId } = useApp.getState().cloud!;
  return `${hubUrl}/api/v1/tenants/${encodeURIComponent(tenantId)}${path}`;
}

/** A tenant API request with the session token; a rejected token signs out. */
export async function tenantRequest(path: string, init?: RequestInit): Promise<unknown> {
  const response = await fetch(tenantUrl(path), {
    ...init,
    headers: { Authorization: `Bearer ${useApp.getState().cloud!.token}` },
  });
  if (response.status === 401) {
    signOut();
    throw new Error("Pandar Hub: the session expired. Sign in again from the Cloud menu.");
  }
  return hubJson(response);
}

export async function listPrinters(): Promise<HubPrinter[]> {
  return ((await tenantRequest("/printers")) as { printers: HubPrinter[] }).printers;
}
