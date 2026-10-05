/** One `Option` of a Tab.cpp line; `index` is the vector slot (`get_option(key, idx)`). */
export interface LayoutOption {
  key: string;
  index?: number;
  /** Tab.cpp `opt.label` override of `ConfigOptionDef::label`. */
  label?: string;
}

/** A row of a `ConfigOptionsGroup` (`OptionsGroup::append_line` / `append_separator`). */
export type LayoutRow =
  | ({ kind: "option"; full?: boolean; code?: boolean } & LayoutOption)
  | { kind: "line"; label: string; options: LayoutOption[] }
  | { kind: "separator" }
  | ({ kind: "widget" } & LayoutOption);

export interface LayoutGroup {
  title: string;
  icon?: string;
  rows: LayoutRow[];
}

export interface LayoutPage {
  title: string;
  groups: LayoutGroup[];
}
