import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";
import { expiresAt, fakeHub, hub, signInDialog, token } from "./pandarHub";

// The Cloud menu's Pandar sign-in and the Devices tab (docs/architecture/web-pandar-print-host.md).
const artifacts = fileURLToPath(new URL("../../target/web-e2e/pandar-cloud/", import.meta.url));
const idle = {
  id: "p-bambu",
  name: "Bambu P1S",
  model: "P1S",
  printer_family: "bambu_lab",
  status: "online",
  nozzle_temperatures: [{ label: null, current_celsius: "25", target_celsius: "0" }],
  bed_temperature_celsius: "24",
  bed_target_temperature_celsius: "0",
  chamber_temperature_celsius: null,
  print: { gcode_state: "IDLE", subtask_name: null, progress_percent: null, remaining_time_minutes: null },
};
const voron = {
  id: "p-klipper",
  name: "Voron",
  model: null,
  printer_family: "moonraker",
  status: "offline",
  nozzle_temperatures: [],
  bed_temperature_celsius: null,
  print: null,
};
const printing = {
  ...idle,
  nozzle_temperatures: [{ label: null, current_celsius: "219", target_celsius: "220" }],
  bed_temperature_celsius: "55",
  bed_target_temperature_celsius: "55",
  print: {
    gcode_state: "RUNNING",
    subtask_name: "benchy",
    progress_percent: 42,
    remaining_time_minutes: 75,
    current_layer: 12,
    total_layers: 80,
  },
};

const cloudMenu = (page: Page) => page.getByRole("button", { name: "Cloud" });
const devicesTab = (page: Page) => page.getByRole("link", { name: "Devices" });
const storedCloud = (page: Page) => page.evaluate(() => JSON.parse(localStorage.getItem("ares.config")!).cloud);

test("signs in to Pandar Cloud and shows its printers live in the Devices tab", async ({ page }) => {
  mkdirSync(artifacts, { recursive: true });
  const hubState = await fakeHub(page.context(), [idle, voron]);
  await page.goto("/#/devices");
  // Signed out: no Devices tab, and its route falls back to Prepare.
  await expect(page).toHaveURL(/#\/prepare$/);
  await expect(devicesTab(page)).toHaveCount(0);

  await cloudMenu(page).click();
  await page.getByRole("menuitem", { name: "Sign in…" }).click();
  await signInDialog(page);
  const session = { token, expiresAt, tenantId: "t1", tenantName: "Workshop", userName: "Ada" };
  expect(await storedCloud(page)).toEqual({ hubUrl: hub, session });

  await devicesTab(page).click();
  await expect(page).toHaveURL(/#\/devices$/);
  const bambu = page.getByRole("article", { name: "Bambu P1S" });
  await expect(bambu).toContainText("online");
  await expect(bambu).toContainText("Nozzle 125 °C");
  await expect(bambu).toContainText("Bed24 °C");
  await expect(bambu.getByRole("progressbar")).toHaveCount(0);
  await expect(page.getByRole("article", { name: "Voron" })).toContainText("offline");
  await expect.poll(() => hubState.sockets.map(({ ticket }) => ticket)).toEqual(["events-1"]);
  const signedIn = await page.screenshot();

  // A printer_snapshot replaces the printer it names.
  hubState.sockets[0]!.socket.send(JSON.stringify({ type: "printer_snapshot", printer: printing }));
  await expect(bambu).toContainText("Nozzle 1219 / 220 °C");
  await expect(bambu).toContainText("benchy · Layer 12/80 · 1 h 15 min left");
  await expect(bambu.getByRole("progressbar", { name: "Bambu P1S progress" })).toHaveAttribute("value", "42");
  const live = await page.screenshot();

  // The session outlives a reload.
  await page.reload();
  await expect(page.getByRole("article", { name: "Bambu P1S" })).toBeVisible();
  await expect.poll(() => hubState.sockets.length).toBe(2);

  await cloudMenu(page).click();
  await expect(page.getByRole("menu", { name: "Cloud" })).toContainText("Ada · Workshop");
  await page.getByRole("menuitem", { name: "Sign out" }).click();
  await expect(page).toHaveURL(/#\/prepare$/);
  await expect(devicesTab(page)).toHaveCount(0);
  expect(await storedCloud(page)).toEqual({ hubUrl: hub });

  // The next sign-in offers the last Hub.
  await cloudMenu(page).click();
  await page.getByRole("menuitem", { name: "Sign in…" }).click();
  await expect(page.getByRole("textbox", { name: "Hub URL" })).toHaveValue(hub);

  writeFileSync(`${artifacts}signed-in.png`, signedIn);
  writeFileSync(`${artifacts}live.png`, live);
  writeFileSync(
    `${artifacts}session.json`,
    JSON.stringify({ signedIn: session, sockets: hubState.sockets.map(({ ticket }) => ticket) }, null, 2),
  );
});
