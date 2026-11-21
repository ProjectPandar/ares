import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, type Download, type Locator, type Page } from "@playwright/test";
import { test } from "./failureDiagnostics";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const artifacts = fileURLToPath(new URL("../../target/web-e2e/file-menu/", import.meta.url));
const PROJECT = `${repositoryRoot}tests/parity/orca_cli_ender3.3mf`;
const PRESET = `${repositoryRoot}tests/presets/My X1C 0.4 nozzle.orca_printer`;
const USER_PRINTER = "My X1C 0.4 nozzle";

/** Label, Ctrl key and the reason of an item the web cannot perform yet; "-" is a separator. */
type Row = [label: string, key: string, reason: string] | "-";
const WRITER = "Needs the 3MF project writer.";
// MainFrame.cpp:3030-3183 and :3741-3779 in the order OrcaSlicer renders on macOS.
const FILE_ROWS: Row[] = [
  ["New Window", "", ""],
  ["New Project", "", ""],
  ["Open Project…", "O", ""],
  ["Recent files", "", "Recent files are not kept in the browser yet."],
  ["Save Project", "", WRITER],
  ["Save Project as…", "", WRITER],
  "-",
  ["Import", "", ""],
  ["Export", "", ""],
  "-",
  ["Sync Presets", "", "Cloud preset sync is not available in the browser."],
  "-",
  ["Preset Bundle", "", "The preset bundle dialog is not available yet."],
];
const IMPORT_ROWS: Row[] = [
  ["Import 3MF/STL…", "I", ""],
  ["Import ZIP Archive…", "", "ZIP archives cannot be imported yet."],
  ["Import Configs…", "", ""],
];
const STL = "STL export is not available yet.";
const DRC = "DRC export is not available yet.";
const EXPORT_ROWS: Row[] = [
  ["Export all objects as one STL…", "", STL],
  ["Export all objects as STLs…", "", STL],
  ["Export all objects as one DRC…", "", DRC],
  ["Export all objects as DRCs…", "", DRC],
  ["Export Generic 3MF…", "", WRITER],
  ["Export plate sliced file…", "", WRITER],
  ["Export all plate sliced file…", "", WRITER],
  ["Export G-code…", "G", ""],
  ["Export toolpaths as OBJ…", "", "Toolpath OBJ export is not available yet."],
  ["Export Preset Bundle…", "", "Preset bundle export is not available yet."],
];
const SUBMENUS = ["Recent files", "Import", "Export"];
/** Items enabled with the engine ready and nothing sliced. */
const ENABLED = ["New Window", "New Project", "Open Project…", "Import", "Export", "Import 3MF/STL…", "Import Configs…"];

/** The rows a menu should show, with the platform's Ctrl label and `aria-keyshortcuts` modifier. */
const expected = (rows: Row[], enabled: string[], mac: boolean) => rows.map((row) => row === "-" ? "-" : {
  label: row[0],
  enabled: enabled.includes(row[0]),
  shortcut: row[1] && `${mac ? "⌘+" : "Ctrl+"}${row[1]}`,
  keys: row[1] && `${mac ? "Meta" : "Control"}+${row[1]}`,
  reason: row[2],
  submenu: SUBMENUS.includes(row[0]),
});

/** A menu's own rows, in order, as a reader gets them. */
const readMenu = (menu: Locator) => menu.evaluate((root) =>
  [...root.querySelectorAll(":scope > [role=menuitem], :scope > [role=separator], :scope > [role=none] > [role=menuitem]")]
    .map((item) => {
      if (item.getAttribute("role") === "separator") return "-";
      const description = item.getAttribute("aria-describedby");
      return {
        label: item.getAttribute("aria-label"),
        enabled: item.getAttribute("aria-disabled") !== "true",
        shortcut: item.querySelector(".menu-shortcut")?.textContent ?? "",
        keys: item.getAttribute("aria-keyshortcuts") ?? "",
        reason: (description && document.getElementById(description)?.textContent) ?? "",
        submenu: item.getAttribute("aria-haspopup") === "menu",
      };
    }));

const sha256 = (download: Download) => download.path().then((path) => createHash("sha256").update(readFileSync(path)).digest("hex"));

/** What a load shows: the window title, the object list and the status line. */
const loaded = async (page: Page) => ({
  title: await page.locator(".titlebar-title").textContent(),
  objects: await page.getByTestId("object-list").getByRole("treeitem").allTextContents(),
  status: await page.getByTestId("status").textContent(),
});

test("the File menu lists upstream's rows and runs import, open, export, new project and new window", async ({ page, context }) => {
  test.setTimeout(300_000);
  rmSync(artifacts, { recursive: true, force: true });
  mkdirSync(artifacts, { recursive: true });
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const downloads: string[] = [];
  page.on("download", (download) => downloads.push(download.suggestedFilename()));

  await page.goto("/");
  await expect(page.locator(".engine-dot.ready")).toBeVisible({ timeout: 60_000 });
  const mac = await page.evaluate(() => /Mac|iPhone|iPad/.test(navigator.userAgent));
  const status = page.getByTestId("status");
  const fileButton = page.getByRole("button", { name: "File", exact: true });
  const menu = (name: string) => page.getByRole("menu", { name, exact: true });
  const item = (parent: string, name: string) => menu(parent).getByRole("menuitem", { name, exact: true });
  /** Picks an item, through its submenu when it has one. */
  const pick = async (path: [string] | [string, string]) => {
    await fileButton.click();
    if (path.length === 2) await item("File", path[0]).hover();
    await item(path.length === 2 ? path[0] : "File", path.at(-1)!).click();
  };
  const chooser = async (path: [string] | [string, string]) =>
    (await Promise.all([page.waitForEvent("filechooser"), pick(path)]))[0];
  const readAll = async () => {
    await fileButton.click();
    const file = await readMenu(menu("File"));
    await item("File", "Import").hover();
    const imports = await readMenu(menu("Import"));
    await item("File", "Export").hover();
    await expect(menu("Import")).toHaveCount(0);
    const exports = await readMenu(menu("Export"));
    return { file, import: imports, export: exports };
  };

  // Keyboard: ArrowDown opens on the first enabled item, separators are skipped, a submenu is entered
  // with ArrowRight or Enter and left with ArrowLeft or Escape; Escape then closes the menu.
  await expect(page.getByRole("button", { name: "Import", exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Import Configs", exact: true })).toHaveCount(0);
  await expect(fileButton).toHaveAttribute("aria-haspopup", "menu");
  await fileButton.focus();
  await page.keyboard.press("ArrowDown");
  await expect(fileButton).toHaveAttribute("aria-expanded", "true");
  await expect(item("File", "New Window")).toBeFocused();
  for (let step = 0; step < 6; step += 1) await page.keyboard.press("ArrowDown");
  await expect(item("File", "Import")).toBeFocused();
  await page.keyboard.press("ArrowRight");
  await expect(item("Import", "Import 3MF/STL…")).toBeFocused();
  await page.keyboard.press("ArrowDown");
  const zip = item("Import", "Import ZIP Archive…");
  await expect(zip).toBeFocused();
  await expect(zip).toHaveAccessibleDescription("ZIP archives cannot be imported yet.");
  await page.keyboard.press("ArrowLeft");
  await expect(menu("Import")).toHaveCount(0);
  await expect(item("File", "Import")).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  // Nothing is sliced, so no Export item is enabled: the first one takes the focus.
  await expect(item("Export", "Export all objects as one STL…")).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(menu("Export")).toHaveCount(0);
  await expect(item("File", "Export")).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(menu("File")).toHaveCount(0);
  await expect(fileButton).toBeFocused();
  await expect(fileButton).toHaveAttribute("aria-expanded", "false");

  // A press outside closes every level.
  await fileButton.click();
  await item("File", "Export").hover();
  await status.click();
  await expect(menu("File")).toHaveCount(0);
  await expect(menu("Export")).toHaveCount(0);

  const empty = await readAll();
  await page.keyboard.press("Escape");
  await page.keyboard.press("Escape");
  expect(empty).toEqual({
    file: expected(FILE_ROWS, ENABLED, mac),
    import: expected(IMPORT_ROWS, ENABLED, mac),
    export: expected(EXPORT_ROWS, ENABLED, mac),
  });

  // The project loaded through the input the File menu opens; slice, then export from the side button.
  await page.getByTestId("import-input").setInputFiles(PROJECT);
  await expect(status).toHaveText("Ready to slice", { timeout: 60_000 });
  const project = await loaded(page);
  expect(project.objects.length).toBeGreaterThan(1);
  await page.getByRole("button", { name: "Slice plate", exact: true }).click();
  await expect(status).toContainText("Slice complete", { timeout: 240_000 });
  const [button] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export G-code file", exact: true }).click(),
  ]);

  const sliced = await readAll();
  await page.screenshot({ path: `${artifacts}export-menu.png` });
  await page.keyboard.press("Escape");
  await page.keyboard.press("Escape");
  expect(sliced).toEqual({
    file: expected(FILE_ROWS, [...ENABLED, "Export G-code…"], mac),
    import: expected(IMPORT_ROWS, ENABLED, mac),
    export: expected(EXPORT_ROWS, [...ENABLED, "Export G-code…"], mac),
  });

  // File › Export › Export G-code… saves what the side button saves.
  const [fromMenu] = await Promise.all([page.waitForEvent("download"), pick(["Export", "Export G-code…"])]);
  await expect(menu("File")).toHaveCount(0);
  expect(fromMenu.suggestedFilename()).toBe(button.suggestedFilename());
  expect(await sha256(fromMenu)).toBe(await sha256(button));

  // A disabled item keeps the menu open and does nothing; a click reaches it although Playwright
  // treats it as not enabled.
  await fileButton.click();
  await item("File", "Save Project").click({ force: true });
  await expect(menu("File")).toBeVisible();
  await item("File", "Export").hover();
  await item("Export", "Export plate sliced file…").click({ force: true });
  await expect(menu("Export")).toBeVisible();
  await status.click();
  expect(downloads).toEqual([button.suggestedFilename(), fromMenu.suggestedFilename()]);

  // Import Configs… opens the preset input with its accepted types.
  const configs = await chooser(["Import", "Import Configs…"]);
  expect(await configs.element().getAttribute("data-testid")).toBe("preset-import-input");
  expect(configs.isMultiple()).toBe(true);
  await configs.setFiles(PRESET);
  const printer = page.getByRole("button", { name: "Printer preset", exact: true });
  const userPrinter = page.getByRole("listbox", { name: "Printer preset", exact: true })
    .getByRole("group", { name: "User presets" }).getByRole("option", { name: USER_PRINTER, exact: true });
  await expect.poll(async () => {
    await printer.click();
    const shown = await userPrinter.count();
    await page.keyboard.press("Escape");
    return shown;
  }, { timeout: 30_000 }).toBe(1);
  await printer.click();
  await userPrinter.click();
  await expect(printer).toHaveText(USER_PRINTER);

  // New Project asks first: Cancel keeps the project; Discard empties the plate, keeps the selected
  // presets and shows Prepare.
  await page.getByRole("link", { name: "Preview" }).click();
  await expect(page).toHaveURL(/#\/preview$/);
  const discard = page.getByRole("dialog", { name: "Discard the current project?" });
  await pick(["New Project"]);
  await expect(discard).toBeVisible();
  await discard.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(discard).toHaveCount(0);
  expect((await loaded(page)).objects).toEqual(project.objects);
  await expect(page.locator(".titlebar-title")).toHaveText(project.title!);
  await pick(["New Project"]);
  await discard.getByRole("button", { name: "Discard", exact: true }).click();
  await expect(discard).toHaveCount(0);
  await expect(page).toHaveURL(/#\/prepare$/);
  await expect(page.locator(".titlebar-title")).toHaveText("Ares");
  await expect(status).toHaveText("Import a 3MF project or STL model to begin");
  await expect(page.getByText("No objects", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Export G-code file", exact: true })).toBeDisabled();
  await expect(printer).toHaveText(USER_PRINTER);

  // Open Project… takes one 3MF and loads it as the input path did.
  const open = await chooser(["Open Project…"]);
  expect(await open.element().getAttribute("data-testid")).toBe("project-input");
  expect(await open.element().getAttribute("accept")).toBe(".3mf");
  expect(open.isMultiple()).toBe(false);
  await open.setFiles(PROJECT);
  await expect(status).toHaveText("Ready to slice", { timeout: 60_000 });
  expect(await loaded(page)).toEqual(project);

  // Ctrl+O opens the same picker.
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
  const [shortcut] = await Promise.all([page.waitForEvent("filechooser"), page.keyboard.press("ControlOrMeta+o")]);
  expect(await shortcut.element().getAttribute("data-testid")).toBe("project-input");

  // Import 3MF/STL… opens the import input.
  await pick(["New Project"]);
  await discard.getByRole("button", { name: "Discard", exact: true }).click();
  await expect(page.getByText("No objects", { exact: true })).toBeVisible();
  const imported = await chooser(["Import", "Import 3MF/STL…"]);
  expect(await imported.element().getAttribute("data-testid")).toBe("import-input");
  expect(imported.isMultiple()).toBe(true);
  await imported.setFiles(PROJECT);
  await expect(status).toHaveText("Ready to slice", { timeout: 60_000 });
  expect(await loaded(page)).toEqual(project);

  // New Window opens the app in a second page.
  const [second] = await Promise.all([context.waitForEvent("page"), pick(["New Window"])]);
  const app = (url: string) => { const { origin, pathname } = new URL(url); return origin + pathname; };
  await expect.poll(() => app(second.url())).toBe(app(page.url()));
  await second.close();

  writeFileSync(`${artifacts}menu.json`, `${JSON.stringify({
    empty,
    sliced,
    gcode: { name: button.suggestedFilename(), sha256: await sha256(button), menuSha256: await sha256(fromMenu) },
    project,
  }, null, 2)}\n`);
  expect(errors).toEqual([]);
});
