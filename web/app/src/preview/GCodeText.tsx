import { useMemo, useState } from "react";

const ROW_HEIGHT = 18;
const OVERSCAN = 20;

/** Virtualized G-code listing (GCodeViewer's G-code window). */
export function GCodeText({ gcode }: { gcode: Uint8Array }) {
  const lines = useMemo(() => new TextDecoder().decode(gcode).split("\n"), [gcode]);
  const [scrollTop, setScrollTop] = useState(0);
  const [height, setHeight] = useState(800);
  const first = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const last = Math.min(lines.length, Math.ceil((scrollTop + height) / ROW_HEIGHT) + OVERSCAN);
  const digits = String(lines.length).length;
  return (
    <aside className="gcode-panel" data-testid="gcode-panel">
      <header className="gcode-panel-header">
        <span>G-code</span>
        <span className="gcode-panel-count">{lines.length.toLocaleString()} lines</span>
      </header>
      <div className="gcode-scroll"
        ref={(element) => {
          if (element && element.clientHeight !== height) setHeight(element.clientHeight);
        }}
        onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}>
        <div style={{ height: lines.length * ROW_HEIGHT, position: "relative" }}>
          {lines.slice(first, last).map((line, offset) => {
            const index = first + offset;
            const comment = line.indexOf(";");
            return (
              <div className="gcode-line" key={index} style={{ top: index * ROW_HEIGHT }}>
                <span className="gcode-ln">{String(index + 1).padStart(digits, " ")}</span>
                {comment === -1 ? (
                  <span className="gcode-cmd">{line}</span>
                ) : (
                  <>
                    <span className="gcode-cmd">{line.slice(0, comment)}</span>
                    <span className="gcode-comment">{line.slice(comment)}</span>
                  </>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </aside>
  );
}
