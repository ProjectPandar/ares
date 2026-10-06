import { useEffect, useMemo, useRef, useState } from "react";

const ROW_HEIGHT = 18;
const PADDING = 8;
const MAX_LENGTH = 55;

/**
 * `GCodeWindow::render` `update_lines`: cuts lines past 55 characters to 52
 * plus "...", keeps the last `;` comment and splits the command from its
 * parameters (`boost::split`, `token_compress_on`).
 */
function splitLine(text: string) {
  const cut = text.length > MAX_LENGTH ? `${text.slice(0, MAX_LENGTH - 3)}...` : text;
  const tokens = cut.split(/;+/);
  const comment = tokens.length > 1 ? `;${tokens.at(-1)}` : "";
  const [command, ...parameters] = tokens[0] ? tokens[0].split(/ +/) : [""];
  return { command, parameters: parameters.map((word) => ` ${word}`).join(""), comment };
}

/**
 * GCodeViewer's G-code window (`GCodeViewer::SequentialView::GCodeWindow::render`,
 * `GCodeViewer.cpp:764-947`): as many lines as fit, centred on the current
 * move's 1-based G-code line `current`, which is framed. `GCodeViewer::render`
 * passes `get_current_vertex().gcode_id` (`GCodeViewer.cpp:1601`); there is no
 * window without one (`curr_line_id == 0`).
 */
export function GCodeWindow({ gcode, current }: { gcode: Uint8Array; current: number }) {
  const lines = useMemo(() => {
    const all = new TextDecoder().decode(gcode).split("\n");
    if (all.at(-1) === "") all.pop();
    return all;
  }, [gcode]);
  const root = useRef<HTMLElement>(null);
  const [height, setHeight] = useState(0);
  useEffect(() => {
    const observer = new ResizeObserver(([entry]) => setHeight(entry.contentRect.height));
    observer.observe(root.current!);
    return () => observer.disconnect();
  }, []);

  const count = Math.max(0, Math.floor((height - 2 * PADDING) / ROW_HEIGHT));
  const half = Math.floor(count / 2);
  let start = current >= half ? current - half : 0;
  let end = start + count - 1;
  // Upstream clamps to `lines_ends.size() - 1`, so the last line is never shown.
  if (end >= lines.length) {
    end = lines.length - 1;
    start = end - count + 1;
  }
  start = Math.max(1, start);
  const digits = String(end).length;
  const rows = [];
  for (let id = start; id <= end; id++) {
    const { command, parameters, comment } = splitLine(lines[id - 1]);
    rows.push(
      <div className={id === current ? "gcode-line current" : "gcode-line"} key={id}
        aria-current={id === current ? "true" : undefined}>
        <span className="gcode-ln">{String(id).padStart(digits, " ")}</span>
        <span className="gcode-text">
          <span className="gcode-cmd">{command}</span>
          <span className="gcode-params">{parameters}</span>
          <span className="gcode-comment">{comment}</span>
        </span>
      </div>,
    );
  }
  return (
    <aside className="gcode-panel" data-testid="gcode-panel" aria-label="G-code" ref={root}>
      <div className="gcode-lines">{rows}</div>
    </aside>
  );
}
