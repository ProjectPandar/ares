import type { Locator } from "@playwright/test";

/**
 * The dialog anatomy of design/ares-slicer-dialogs/assets/dialogs.css, by its own class names: every
 * part the app renders with the design's vocabulary is read in the artboard and in the app alike.
 * A part the app renders with its own class names (the settings pages share the sidebar's controls)
 * names the app's selector in the opening's `appSelectors`.
 */
export interface Part {
  part: string;
  selector: string;
  /** A wrapper of the artboard that carries no look: its absence from the app is no difference. */
  optional?: boolean;
}

export const PARTS: Part[] = [
  { part: "dialog", selector: "dialog.dialog[open], .popup" },
  { part: "header", selector: ".dialog-header" },
  { part: "heading", selector: ".dialog-heading h1" },
  { part: "description", selector: ".dialog-heading p" },
  { part: "step", selector: ".step" },
  { part: "step-count", selector: ".step b" },
  { part: "close", selector: ".button.close" },
  { part: "body", selector: ".dialog-body" },
  { part: "stack", selector: ".stack" },
  { part: "paragraph", selector: ".dialog-body > p" },
  { part: "footer", selector: ".dialog-footer" },
  { part: "summary", selector: ".dialog-footer .summary" },
  { part: "button", selector: ".dialog-footer .button:not(.primary, .danger, .close)" },
  { part: "primary", selector: ".button.primary" },
  { part: "danger", selector: ".button.danger" },
  { part: "field", selector: ".field" },
  { part: "field-label", selector: ".field > span:first-child" },
  { part: "input", selector: ".field :is(input, select, textarea)" },
  { part: "search", selector: ".search" },
  { part: "check", selector: ".check" },
  { part: "check-input", selector: ".check input" },
  { part: "check-detail", selector: ".check-detail" },
  { part: "notice", selector: ".notice" },
  { part: "spinner", selector: ".spinner" },
  { part: "muted", selector: ".muted" },
  { part: "frame", selector: ".frame" },
  { part: "frame-head", selector: ".frame-head" },
  { part: "frame-title", selector: ".frame-head :is(h2, h3)", optional: true },
  { part: "frame-content", selector: ".frame-content" },
  { part: "catalog", selector: ".catalog" },
  { part: "catalog-actions", selector: ".catalog-actions" },
  { part: "catalog-action", selector: ".catalog-actions .button" },
  { part: "wizard", selector: ".wizard" },
  { part: "vendor-nav", selector: ".vendor-nav" },
  { part: "vendor-link", selector: ".vendor-nav :is(a, button)" },
  { part: "models", selector: ".models" },
  { part: "model", selector: ".model:has(input:checked)" },
  { part: "model-unchecked", selector: ".model:not(:has(input:checked))" },
  { part: "model-cover", selector: ".model img" },
  { part: "tabs", selector: ".settings-tabs" },
  { part: "tab", selector: '.settings-tabs [role="tab"][aria-selected="false"]' },
  { part: "tab-active", selector: '.settings-tabs [role="tab"][aria-selected="true"]' },
  { part: "tabpanel", selector: '[role="tabpanel"]:not([hidden])', optional: true },
  { part: "setting-field", selector: ".settings .field" },
  { part: "setting-input", selector: ".settings .field :is(input, select)" },
  { part: "shortcuts-list", selector: ".shortcuts-list" },
  { part: "shortcut-row", selector: ".shortcuts-list > div" },
  { part: "shortcut-description", selector: ".shortcuts-list dt" },
  { part: "shortcut-keys", selector: ".shortcuts-list dd" },
  { part: "kbd", selector: "kbd" },
  { part: "about-mark", selector: ".about-mark" },
  { part: "about-identity", selector: ".about-identity" },
  { part: "about-name", selector: ".about-identity strong" },
  { part: "spacing-row", selector: ".spacing-row" },
  { part: "range", selector: '.spacing-row input[type="range"]' },
];

/** The look of a part; its box follows its content and is reported, not judged. */
export const PROPERTIES = [
  "display",
  "position",
  "flex-direction",
  "flex-wrap",
  "align-items",
  "align-content",
  "justify-content",
  "gap",
  "min-width",
  "min-height",
  "flex",
  "overflow-x",
  "overflow-y",
  "padding-top",
  "padding-right",
  "padding-bottom",
  "padding-left",
  "margin-top",
  "margin-right",
  "margin-bottom",
  "margin-left",
  "border-top-width",
  "border-right-width",
  "border-bottom-width",
  "border-left-width",
  "border-top-style",
  "border-top-color",
  "border-radius",
  "background-color",
  "color",
  "opacity",
  "box-shadow",
  "font-family",
  "font-size",
  "font-weight",
  "line-height",
  "letter-spacing",
  "font-variant-numeric",
  "text-align",
  "white-space",
  "accent-color",
] as const;

export interface PartStyles {
  properties: Record<(typeof PROPERTIES)[number], string>;
  /** Border-box size in CSS px. */
  size: { width: number; height: number };
}

export type Styles = Record<string, PartStyles>;

export const OPTIONAL_PARTS = new Set(PARTS.filter(({ optional }) => optional).map(({ part }) => part));

/** Computed styles of every part `root` holds, with `selectors` in place of the design's on the app's side. */
export async function readStyles(root: Locator, selectors: Record<string, string> = {}): Promise<Styles> {
  return root.evaluate(
    (element, { parts, properties, overrides }) => {
      const styles: Record<string, { properties: Record<string, string>; size: { width: number; height: number } }> =
        {};
      for (const { part, selector } of parts) {
        const query = overrides[part] ?? selector;
        // A nested dialog is compared on its own, not as part of the one under it.
        const target: Element | null =
          element.tagName === "DIALOG" ? (element.matches(query) ? element : element.querySelector(query)) : null;
        if (!target || (element.tagName === "DIALOG" && target.closest("dialog") !== element)) continue;
        const style = getComputedStyle(target);
        const box = target.getBoundingClientRect();
        styles[part] = {
          properties: Object.fromEntries(properties.map((property) => [property, style.getPropertyValue(property)])),
          size: { width: box.width, height: box.height },
        };
      }
      return styles;
    },
    { parts: PARTS, properties: PROPERTIES, overrides: selectors },
  );
}
