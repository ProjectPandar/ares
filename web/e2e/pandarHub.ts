import { createHash } from "node:crypto";
import { type BrowserContext, expect, type Page, type Request, type WebSocketRoute } from "@playwright/test";

/** A fake Pandar Hub tenant `t1` for the Cloud menu (docs/architecture/web-pandar-print-host.md). */
export const hub = "http://hub.pandar.test";
export const token = "pandar_web_token";
export const expiresAt = "2099-01-01T00:00:00Z";
const app = "http://127.0.0.1:4174";
const cors = {
  "access-control-allow-origin": app,
  "access-control-allow-methods": "GET, POST, OPTIONS",
  "access-control-allow-headers": "authorization, content-type",
};

export interface FakeHub {
  /** Each job upload. */
  jobs: Request[];
  /** Each printer-event socket the app opened, by ticket. */
  sockets: { ticket: string; socket: WebSocketRoute }[];
}

/**
 * Routes the Hub for every page of `context`, the sign-in popup included: `/web-sign-in` redirects
 * at once to the app's callback with a ticket, and the exchange checks the PKCE verifier against
 * the challenge it was issued for.
 */
export async function fakeHub(context: BrowserContext, printers: object[]): Promise<FakeHub> {
  const hubState: FakeHub = { jobs: [], sockets: [] };
  const challenges = new Map<string, string>();
  let tickets = 0;
  await context.route(`${hub}/**`, async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const path = url.pathname;
    if (path === "/web-sign-in") {
      const redirect = new URL(url.searchParams.get("redirect_url")!);
      if (redirect.href !== `${app}/cloud-callback.html`) return route.fulfill({ status: 400, body: "redirect_url" });
      const ticket = `pandar_web_ticket_${++tickets}`;
      challenges.set(ticket, url.searchParams.get("code_challenge")!);
      redirect.search = new URLSearchParams({ ticket, state: url.searchParams.get("state")! }).toString();
      return route.fulfill({ status: 302, headers: { location: redirect.href } });
    }
    if (request.method() === "OPTIONS") return route.fulfill({ status: 204, headers: cors });
    if (request.method() === "POST" && path === "/api/v1/web/login-tickets/exchange") {
      const { ticket, code_verifier } = request.postDataJSON() as { ticket: string; code_verifier: string };
      const challenge = createHash("sha256").update(code_verifier).digest("base64url");
      if (challenges.get(ticket) !== challenge)
        return route.fulfill({ status: 401, headers: cors, json: { error: "invalid_plugin_ticket" } });
      challenges.delete(ticket);
      return route.fulfill({
        headers: cors,
        json: {
          token,
          expires_at: expiresAt,
          profile: { user_id: "u1", user_name: "Ada", tenant_id: "t1", tenant_name: "Workshop" },
        },
      });
    }
    if (request.headers().authorization !== `Bearer ${token}`)
      return route.fulfill({ status: 401, headers: cors, json: { error: "unauthorized" } });
    if (request.method() === "GET" && path === "/api/v1/tenants/t1/printers")
      return route.fulfill({ headers: cors, json: { printers } });
    if (request.method() === "POST" && path === "/api/v1/tenants/t1/printer-events/tickets")
      return route.fulfill({
        status: 201,
        headers: cors,
        json: { ticket: `events-${hubState.sockets.length + 1}`, expires_at: expiresAt },
      });
    if (request.method() === "POST" && /^\/api\/v1\/tenants\/t1\/printers\/[^/]+\/jobs$/.test(path)) {
      hubState.jobs.push(request);
      return route.fulfill({ status: 201, headers: cors, json: { id: `job-${hubState.jobs.length}` } });
    }
    return route.fulfill({ status: 404, headers: cors, json: { error: "not_found" } });
  });
  await context.routeWebSocket(/^ws:\/\/hub\.pandar\.test\/api\/v1\/tenants\/t1\/printer-events\?/, (socket) => {
    hubState.sockets.push({ ticket: new URL(socket.url()).searchParams.get("ticket")!, socket });
  });
  return hubState;
}

/** Fills the Cloud sign-in dialog and waits for the popup's round trip. */
export async function signInDialog(page: Page) {
  const dialog = page.getByRole("dialog", { name: "Sign in to Pandar Cloud" });
  await dialog.getByRole("textbox", { name: "Hub URL" }).fill(`${hub}/`);
  await Promise.all([page.waitForEvent("popup"), dialog.getByRole("button", { name: "Sign in" }).click()]);
  await expect(dialog).toBeHidden();
}
