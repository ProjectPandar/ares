import { expect, type BrowserContext, type Page, type Route } from "@playwright/test";
import { expiresAt, token } from "../pandarHub";
import { config, dialog, importFile, parity, ready, seed, slice } from "./app";
import type { Opening } from "./states";

/** The Pandar Hub at the URL the artboards show; a failed ticket exchange, a sign-in page that never returns, or a session. */
async function hub(context: BrowserContext, baseURL: string | undefined, outcome: "error" | "hold") {
  const url = "https://pandar.example.com";
  if (!baseURL) throw new Error("The fake Pandar Hub requires a Playwright baseURL");
  const cors = {
    "access-control-allow-origin": new URL(baseURL).origin,
    "access-control-allow-methods": "GET, POST, OPTIONS",
    "access-control-allow-headers": "authorization, content-type",
  };
  const printers = [
    { id: "p-a1-mini", name: "Bambu Lab A1 mini", model: "A1 mini", printer_family: "bambu_lab", status: "Idle" },
    { id: "p-klipper", name: "Generic Klipper Printer", model: null, printer_family: "moonraker", status: "Idle" },
  ];
  await context.route(`${url}/**`, async (route: Route) => {
    const request = route.request();
    const address = new URL(request.url());
    const path = address.pathname;
    if (path === "/web-sign-in" && outcome === "error") {
      const redirect = new URL(address.searchParams.get("redirect_url")!);
      redirect.search = new URLSearchParams({
        ticket: "pandar_web_ticket_1",
        state: address.searchParams.get("state")!,
      }).toString();
      return route.fulfill({ status: 302, headers: { location: redirect.href } });
    }
    if (path === "/web-sign-in")
      return route.fulfill({ contentType: "text/html", body: "<!doctype html><title>Pandar</title>" });
    if (request.method() === "OPTIONS") return route.fulfill({ status: 204, headers: cors });
    if (path === "/api/v1/web/login-tickets/exchange")
      return route.fulfill({
        status: outcome === "error" ? 503 : 200,
        headers: cors,
        json:
          outcome === "error"
            ? { error: "hub_unavailable" }
            : {
                token,
                expires_at: expiresAt,
                profile: { user_id: "u1", user_name: "Ada", tenant_id: "t1", tenant_name: "Workshop" },
              },
      });
    if (request.headers().authorization !== `Bearer ${token}`)
      return route.fulfill({ status: 401, headers: cors, json: { error: "unauthorized" } });
    if (request.method() === "GET" && path === "/api/v1/tenants/t1/printers")
      return route.fulfill({ headers: cors, json: { printers } });
    if (request.method() === "POST" && path === "/api/v1/tenants/t1/printer-events/tickets")
      return route.fulfill({ status: 201, headers: cors, json: { ticket: "events-1", expires_at: expiresAt } });
    if (request.method() === "POST" && /^\/api\/v1\/tenants\/t1\/printers\/[^/]+\/jobs$/.test(path))
      return route.fulfill({ status: 201, headers: cors, json: { id: "job-1" } });
    return route.fulfill({ status: 404, headers: cors, json: { error: "not_found" } });
  });
  return url;
}

const session = { token, expiresAt, tenantId: "t1", tenantName: "Workshop", userName: "Ada" };

async function signInDialog(page: Page, context: BrowserContext, baseURL: string | undefined) {
  const url = await hub(context, baseURL, "hold");
  seed(page, config("light", { cloud: { hubUrl: url } }));
  await ready(page);
  await page.getByRole("button", { name: "Cloud", exact: true }).click();
  await page.getByRole("menuitem", { name: "Sign in…" }).click();
  const sign = dialog(page, "Sign in to Pandar Cloud");
  await expect(sign.getByRole("textbox", { name: "Hub URL" })).toHaveValue(url);
  return sign;
}

/** Opens "Send to Printer" over a sliced plate, signed in to the Hub with a Bambu printer first. */
async function sendDialog(
  page: Page,
  context: BrowserContext,
  baseURL: string | undefined,
  theme: "light" | "dark",
  project: string | null,
) {
  const url = await hub(context, baseURL, "hold");
  seed(page, config(theme, { cloud: { hubUrl: url, session } }));
  await ready(page);
  await importFile(page, project ?? `${parity}cube10.stl`);
  await slice(page);
  const options = page.getByRole("button", { name: "Output options" });
  await options.click();
  await page.getByRole("menuitemradio", { name: "Print plate" }).click();
  await page.getByRole("button", { name: "Print plate", exact: true }).click();
  const send = dialog(page, "Send to Printer");
  await expect(send.getByRole("combobox", { name: "Printer" })).toBeEnabled({ timeout: 30_000 });
  return send;
}

const hubLine = "the Hub line shows the Hub URL with its scheme";

/** The Cloud sign-in and the Send to Printer artboards. */
export const CLOUD_OPENINGS: Record<string, Opening> = {
  "cloud-sign-in": {
    theme: "light",
    notes: [],
    open: async ({ page, context, baseURL }) => signInDialog(page, context, baseURL),
  },
  "cloud-waiting": {
    theme: "light",
    notes: ["the sign-in window is a placeholder page the Hub route holds open"],
    open: async ({ page, context, baseURL }) => {
      const sign = await signInDialog(page, context, baseURL);
      await sign.getByRole("button", { name: "Sign in", exact: true }).click();
      await expect(sign.getByRole("status")).toBeVisible();
      return sign;
    },
  },
  "cloud-error": {
    theme: "dark",
    notes: ["the error is the Hub's answer to the ticket exchange"],
    open: async ({ page, context, baseURL }) => {
      const url = await hub(context, baseURL, "error");
      seed(page, config("dark", { cloud: { hubUrl: url } }));
      await ready(page);
      await page.getByRole("button", { name: "Cloud", exact: true }).click();
      await page.getByRole("menuitem", { name: "Sign in…" }).click();
      const sign = dialog(page, "Sign in to Pandar Cloud");
      await sign.getByRole("button", { name: "Sign in", exact: true }).click();
      await expect(sign.getByRole("alert")).toBeVisible({ timeout: 30_000 });
      return sign;
    },
  },
  "send-to-printer": {
    theme: "light",
    notes: [hubLine],
    open: async ({ page, context, baseURL }) => sendDialog(page, context, baseURL, "light", null),
  },
  "send-blocked": {
    theme: "dark",
    notes: [hubLine],
    open: async ({ page, context, baseURL }) =>
      sendDialog(page, context, baseURL, "dark", `${parity}custom-gcode-overview/two-plates.3mf`),
  },
  "send-sent": {
    theme: "light",
    notes: [hubLine],
    open: async ({ page, context, baseURL }) => {
      const send = await sendDialog(page, context, baseURL, "light", null);
      await send.getByRole("button", { name: "Send", exact: true }).click();
      await expect(send.getByRole("status")).toHaveText("Sent to Bambu Lab A1 mini.", { timeout: 120_000 });
      return send;
    },
  },
  "send-mobile": {
    theme: "light",
    notes: [hubLine],
    open: async ({ page, context, baseURL }) => sendDialog(page, context, baseURL, "light", null),
  },
};
