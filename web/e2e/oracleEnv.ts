import { existsSync } from "node:fs";
import { test } from "@playwright/test";

/**
 * Path of the external oracle named by the environment variable `name`.
 * A set variable whose path does not exist always fails the test. An unset
 * variable fails it outside CI, where a check that did not run is not a pass;
 * CI does not provision the oracles, so there it is reported as skipped and
 * `null` is returned.
 */
export function oraclePath(name: string, oracle: string, source: string): string | null {
  const path = process.env[name];
  if (path && existsSync(path)) return path;
  if (path) throw new Error(`${name}=${path} does not exist: set it to ${oracle} (${source})`);
  if (!process.env.CI) throw new Error(`${name} is unset: the test needs ${oracle} (${source})`);
  test.info().annotations.push({ type: "skip", description: `${name} is unset: ${oracle} not checked` });
  return null;
}
