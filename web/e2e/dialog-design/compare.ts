import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import type { Page } from "@playwright/test";
import { OPTIONAL_PARTS, PROPERTIES, type PartStyles, type Styles } from "./probe";

export interface PartDifference {
  property: string;
  design: string;
  app: string;
}

export interface PartComparison {
  part: string;
  status: "same" | "different" | "missing-in-design" | "missing-in-app";
  differences: PartDifference[];
  /** Border-box sizes when they differ; content, not look, sets them. */
  size?: { design: { width: number; height: number }; app: { width: number; height: number } };
}

export interface StyleComparison {
  parts: PartComparison[];
  /** Parts whose look differs or that only one side renders. */
  mismatches: number;
}

/** A property the app states on purpose where its content needs it (a line break the text carries). */
export interface Ignored {
  part: string;
  property: string;
}

const round = (value: number) => Math.round(value * 2) / 2;

/** Two lengths or plain numbers within a twentieth of a pixel read the same. */
function alike(design: string, app: string) {
  if (design === app) return true;
  const a = /^(-?[\d.]+)px$/.exec(design);
  const b = /^(-?[\d.]+)px$/.exec(app);
  return Boolean(a && b && Math.abs(Number(a[1]) - Number(b[1])) <= 0.05);
}

function comparePart(part: string, design: PartStyles, app: PartStyles, skip: Set<string>): PartComparison {
  const differences: PartDifference[] = [];
  for (const property of PROPERTIES) {
    // The dialog's own margins are its centring, a consequence of its height.
    if (part === "dialog" && property.startsWith("margin")) continue;
    if (skip.has(property)) continue;
    const value = design.properties[property];
    const other = app.properties[property];
    if (!alike(value, other)) differences.push({ property, design: value, app: other });
  }
  const size = {
    design: { width: round(design.size.width), height: round(design.size.height) },
    app: { width: round(app.size.width), height: round(app.size.height) },
  };
  return {
    part,
    status: differences.length ? "different" : "same",
    differences,
    size: size.design.width === size.app.width && size.design.height === size.app.height ? undefined : size,
  };
}

/** The artboard's computed styles beside the app's, part by part and property by property. */
export function diffStyles(design: Styles, app: Styles, ignored: readonly Ignored[] = []): StyleComparison {
  const parts: PartComparison[] = [];
  for (const [part, styles] of Object.entries(design)) {
    if (!(part in app)) {
      if (!OPTIONAL_PARTS.has(part)) parts.push({ part, status: "missing-in-app", differences: [] });
      continue;
    }
    const skip = new Set(ignored.filter((entry) => entry.part === part).map(({ property }) => property));
    parts.push(comparePart(part, styles, app[part]!, skip));
  }
  for (const part of Object.keys(app)) {
    if (!(part in design)) parts.push({ part, status: "missing-in-design", differences: [] });
  }
  return { parts, mismatches: parts.filter(({ status }) => status !== "same").length };
}

export interface PixelComparison {
  width: number;
  height: number;
  compared: number;
  /** Pixels whose largest channel difference exceeds the tolerance. */
  different: number;
  /** Pixels that differ at all, anti-aliasing included. */
  strict: number;
  /** Pixels inside the dialog on one side only. */
  shape: number;
  png: Buffer;
}

/** The dialog crops compared pixel by pixel in a blank page's canvas; a magenta overlay marks the differences. */
export async function pixelDiff(page: Page, design: Buffer, app: Buffer): Promise<PixelComparison> {
  const result = await page.evaluate(
    async ({ a, b }) => {
      const load = (src: string) =>
        new Promise<HTMLImageElement>((resolve, reject) => {
          const image = new Image();
          image.onload = () => resolve(image);
          image.onerror = () => reject(new Error("PNG decode failed"));
          image.src = src;
        });
      const [first, second] = await Promise.all([load(a), load(b)]);
      const width = Math.max(first.width, second.width);
      const height = Math.max(first.height, second.height);
      const read = (image: HTMLImageElement) => {
        const canvas = new OffscreenCanvas(width, height);
        const context = canvas.getContext("2d")!;
        context.drawImage(image, 0, 0);
        return context.getImageData(0, 0, width, height).data;
      };
      const designData = read(first);
      const appData = read(second);
      const out = new Uint8ClampedArray(width * height * 4);
      let compared = 0;
      let different = 0;
      let strict = 0;
      let shape = 0;
      for (let i = 0; i < out.length; i += 4) {
        const inDesign = designData[i + 3] > 0;
        const inApp = appData[i + 3] > 0;
        if (inDesign !== inApp) {
          shape++;
          out.set([255, 196, 0, 255], i);
          continue;
        }
        if (!inDesign) continue;
        compared++;
        let largest = 0;
        for (let channel = 0; channel < 3; channel++)
          largest = Math.max(largest, Math.abs(designData[i + channel] - appData[i + channel]));
        if (largest > 0) strict++;
        if (largest > 24) {
          different++;
          out.set([255, 0, 255, 210], i);
        } else out.set([appData[i]!, appData[i + 1]!, appData[i + 2]!, 255], i);
      }
      const canvas = new OffscreenCanvas(width, height);
      canvas.getContext("2d")!.putImageData(new ImageData(out, width, height), 0, 0);
      const bytes = new Uint8Array(await (await canvas.convertToBlob({ type: "image/png" })).arrayBuffer());
      let binary = "";
      for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
      return { width, height, compared, different, strict, shape, png: btoa(binary) };
    },
    { a: `data:image/png;base64,${design.toString("base64")}`, b: `data:image/png;base64,${app.toString("base64")}` },
  );
  const { png, ...numbers } = result;
  return { ...numbers, png: Buffer.from(png, "base64") };
}

export interface ArtboardResult {
  id: string;
  title: string;
  theme: "light" | "dark";
  viewport: { width: number; height: number };
  styles: StyleComparison;
  pixels: Omit<PixelComparison, "png">;
  /** What differs by content rather than by look, in so many words. */
  notes: string[];
}

export const percent = (part: number, whole: number) => (whole ? ((part / whole) * 100).toFixed(2) : "0.00");

const esc = (text: string) => text.replace(/[&<>"]/g, (c) => `&#${c.charCodeAt(0)};`);

function card(result: ArtboardResult) {
  const failed = result.styles.parts.filter(({ status }) => status !== "same");
  const rows = failed
    .map(({ part, status, differences, size }) => {
      const detail =
        status === "missing-in-app"
          ? "not rendered by the app"
          : status === "missing-in-design"
            ? "app only"
            : [
                ...differences.map(
                  ({ property, design, app }) =>
                    `<li><code>${esc(property)}</code>: ${esc(design)} → <b>${esc(app)}</b></li>`,
                ),
                ...(size
                  ? [`<li>size: ${size.design.width}×${size.design.height} → ${size.app.width}×${size.app.height}</li>`]
                  : []),
              ].join("");
      return `<tr class="${status}"><td><code>${esc(part)}</code></td><td>${esc(status)}</td><td><ul>${detail}</ul></td></tr>`;
    })
    .join("");
  return `<section class="${result.theme}">
    <h2>${esc(result.id)} <small>${esc(result.title)} · ${result.theme} · ${result.viewport.width}×${result.viewport.height}</small></h2>
    <p class="verdict">Look: <b>${failed.length ? `${failed.length} part(s) differ` : "identical computed styles"}</b>
      (of ${result.styles.parts.length} parts). Pixels: <b>${percent(result.pixels.different, result.pixels.compared)}%</b> differ
      (${result.pixels.strict} of ${result.pixels.compared} differ at all, ${result.pixels.shape} on one side only).
      ${result.notes.map((note) => `<span class="note">${esc(note)}</span>`).join("")}</p>
    <div class="shots">
      <figure><img src="${result.id}/reference.png" alt="artboard"><figcaption>Design artboard</figcaption></figure>
      <figure><img src="${result.id}/actual.png" alt="app"><figcaption>App</figcaption></figure>
    </div>
    <div class="shots">
      <figure><img src="${result.id}/dialog-reference.png" alt="design dialog"><figcaption>Design dialog</figcaption></figure>
      <figure><img src="${result.id}/dialog-actual.png" alt="app dialog"><figcaption>App dialog</figcaption></figure>
      <figure><img src="${result.id}/diff.png" alt="diff"><figcaption>Diff: magenta differs, yellow one side only</figcaption></figure>
    </div>
    ${rows ? `<table><thead><tr><th>Part</th><th>Status</th><th>Differences (design → app)</th></tr></thead><tbody>${rows}</tbody></table>` : ""}
  </section>`;
}

/** The gallery of every artboard result written so far. */
export function writeGallery(directory: string, order: string[]) {
  const results = readdirSync(directory, { withFileTypes: true })
    .filter(
      (entry) =>
        entry.isDirectory() && order.includes(entry.name) && existsSync(`${directory}${entry.name}/result.json`),
    )
    .map((entry) => JSON.parse(readFileSync(`${directory}${entry.name}/result.json`, "utf8")) as ArtboardResult)
    .sort((a, b) => order.indexOf(a.id) - order.indexOf(b.id));
  const differing = results.filter((result) => result.styles.mismatches > 0).length;
  writeFileSync(
    `${directory}index.html`,
    `<!doctype html><meta charset="utf-8"><title>Ares dialogs · design parity</title>
<style>
body{margin:24px;background:#f6f7f8;color:#181b20;font:14px/1.5 Inter,"Segoe UI",system-ui,sans-serif}
h1 small{font-weight:400;color:#5d646e}section{background:#fff;border:1px solid #e0e2e6;border-radius:12px;padding:16px 20px;margin:16px 0}
section.dark{background:#1c1e22;color:#f2f3f5;border-color:#373b42}h2{margin:0 0 4px;font-size:17px}h2 small{font-size:12px;font-weight:500;color:#8a919b}
.verdict{margin:0 0 12px}.note{display:inline-block;margin-left:12px;padding:1px 8px;border-radius:5px;background:#8883;color:inherit;font-size:12px}
.shots{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:12px;align-items:start}figure{margin:0}
figure img{width:100%;height:auto;border:1px solid #8885;border-radius:6px;background:repeating-conic-gradient(#9992 0 25%,transparent 0 50%) 0 0/16px 16px}
figcaption{font-size:12px;opacity:.75;margin-top:4px}table{border-collapse:collapse;margin-top:12px;font-size:12px;width:100%}
td,th{border-top:1px solid #8885;padding:4px 10px;text-align:left;vertical-align:top}td ul{margin:0;padding-left:16px}code{font:11px ui-monospace,monospace}
tr.different td:nth-child(2){color:#ae2d35}tr.missing-in-app td:nth-child(2){color:#855211}
</style>
<h1>Ares dialogs · design parity <small>${results.length} artboards, ${differing} with style differences</small></h1>
${results.map(card).join("\n")}
`,
  );
  return { results: results.length, differing };
}
