import { useRef, useSyncExternalStore } from "react";
import type { ViewPreset } from "./camera";
import type { Viewport } from "./Viewport";

type Face = Exclude<ViewPreset, "iso" | "plate">;
const FACES: [Face, string][] = [
  ["top", "Top"],
  ["bottom", "Bottom"],
  ["front", "Front"],
  ["rear", "Rear"],
  ["left", "Left"],
  ["right", "Right"],
];

/** Half the cube edge in the axes' SVG units, which match `--cube-size: 56px`. */
const HALF = 28;
/** The world axes from the front-left-bottom corner, in cube coordinates (CSS: y down, z to the viewer). */
const AXES = [
  { name: "x", direction: [1, 0, 0], length: 56 },
  { name: "y", direction: [0, 0, -1], length: 56 },
  { name: "z", direction: [0, -1, 0], length: 43 },
] as const;

/** The front view's camera turn: the identity CSS transform. */
const FRONT = [1, 0, 0, 0, 0, -1, 0, 1, 0];
const unsubscribed = () => () => undefined;

/**
 * The cube's CSS transform columns (cube x, y, z in screen x, y, z) for the camera's world-to-view
 * rotation `turn` (column-major): cube x is world X, cube y world -Z and cube z world -Y, and the
 * screen's y runs down where the view's runs up.
 */
function cubeColumns(turn: readonly number[]) {
  const [r00, r10, r20, r01, r11, r21, r02, r12, r22] = turn;
  return [
    [r00, -r10, r20],
    [-r02, r12, -r22],
    [-r01, r11, -r21],
  ];
}

/** The screen position of cube point `[x, y, z]` under the cube's `columns`. */
function project([x, y, z]: readonly number[], [cx, cy, cz]: number[][]): [number, number] {
  return [cx[0] * x + cy[0] * y + cz[0] * z, cx[1] * x + cy[1] * y + cz[1] * z];
}

function Axes({ turn }: { turn: number[][] }) {
  const origin = project([-HALF, HALF, HALF], turn);
  return (
    <svg className="view-cube-axes" viewBox="-55 -47 110 94" aria-hidden="true">
      <defs>
        {AXES.map(({ name }) => (
          <marker
            key={name}
            id={`view-cube-arrow-${name}`}
            className={name}
            viewBox="0 0 8 8"
            refX="1"
            refY="4"
            markerWidth="3.2"
            markerHeight="3.2"
            markerUnits="strokeWidth"
            orient="auto"
          >
            <path d="M0 0L8 4L0 8z" fill="currentColor" />
          </marker>
        ))}
      </defs>
      {AXES.map(({ name, direction, length }) => {
        const tip = project(
          direction.map((value, index) => [-HALF, HALF, HALF][index] + value * length),
          turn,
        );
        const [dx, dy] = [tip[0] - origin[0], tip[1] - origin[1]];
        const projected = Math.hypot(dx, dy);
        // An axis along the view direction is a dot.
        if (projected < 1)
          return <circle key={name} className={name} cx={origin[0]} cy={origin[1]} r="3.5" fill="currentColor" />;
        return [
          <line
            key={name}
            className={name}
            x1={origin[0]}
            y1={origin[1]}
            x2={tip[0]}
            y2={tip[1]}
            stroke="currentColor"
            markerEnd={`url(#view-cube-arrow-${name})`}
          />,
          <text
            key={`${name}-label`}
            className={name}
            x={tip[0] + (dx / projected) * 11}
            y={tip[1] + (dy / projected) * 11}
            textAnchor="middle"
            dominantBaseline="central"
            fill="currentColor"
          >
            {name}
          </text>,
        ];
      })}
    </svg>
  );
}

/**
 * The camera view cube at the 3D view's bottom left: a face selects its view (`Camera::select_view`),
 * the top-front-left corner the iso view. The cube turns with the camera, and a drag on it orbits the
 * camera like a canvas drag (`GLCanvas3D::on_mouse` Dragging, `Camera::rotate_on_sphere`).
 */
export function ViewCube({
  viewport,
  active,
  onView,
  onFit,
  canFitObjects,
}: {
  viewport: Viewport | null;
  active: ViewPreset | null;
  onView: (preset: ViewPreset) => void;
  onFit: (target: "plate" | "objects") => void;
  canFitObjects: boolean;
}) {
  const turn = cubeColumns(
    useSyncExternalStore(viewport?.subscribeTurn ?? unsubscribed, () => viewport?.turn ?? FRONT),
  );
  // A press that moves past 3 px drags the cube, and the click that ends it selects no view.
  const drag = useRef<{ x: number; y: number; moved: boolean } | null>(null);
  const iso = (
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M8 2.5l5 2.8v5.6L8 13.5 3 10.9V5.3zM3 5.3l5 2.9 5-2.9M8 8.2v5.3" />
    </svg>
  );
  return (
    <div className="view-cube-nav" role="toolbar" aria-label="Camera view" data-testid="view-toolbar">
      <div
        className="view-cube-stage"
        onPointerDown={(event) => {
          if (event.button === 0) drag.current = { x: event.clientX, y: event.clientY, moved: false };
        }}
        onPointerMove={(event) => {
          const press = drag.current;
          if (!press || !(event.buttons & 1)) return;
          const [dx, dy] = [event.clientX - press.x, event.clientY - press.y];
          if (!press.moved) {
            if (Math.hypot(dx, dy) < 3) return;
            press.moved = true;
            event.currentTarget.setPointerCapture(event.pointerId);
          }
          viewport?.orbit(dx, dy);
          [press.x, press.y] = [event.clientX, event.clientY];
        }}
        onClickCapture={(event) => {
          if (drag.current?.moved) event.stopPropagation();
          drag.current = null;
        }}
      >
        <div
          className="view-cube"
          style={{ transform: `matrix3d(${turn.map((column) => [...column, 0]).flat()},0,0,0,1)` }}
        >
          {FACES.map(([preset, label]) => (
            <button
              key={preset}
              className={`view-cube-face ${preset}`}
              aria-pressed={active === preset}
              title={`${label} view`}
              onClick={() => onView(preset)}
            >
              {label}
            </button>
          ))}
          <button
            className="view-cube-corner"
            aria-pressed={active === "iso"}
            aria-label="Iso view"
            title="Iso view"
            onClick={() => onView("iso")}
          />
        </div>
        <Axes turn={turn} />
      </div>
      <div className="view-cube-actions">
        {/* Touch screens: the corner is too small a target, so the iso view is an action. */}
        <button
          className="view-cube-action iso"
          aria-pressed={active === "iso"}
          aria-label="Iso view"
          title="Iso view"
          onClick={() => onView("iso")}
        >
          {iso}
        </button>
        <button
          className="view-cube-action"
          aria-label="Fit to plate"
          title="Fit to plate"
          onClick={() => onFit("plate")}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.3"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M1.5 4.5v-3h3M11.5 1.5h3v3M14.5 11.5v3h-3M4.5 14.5h-3v-3" />
            <path d="M5 6.5h7l-1 3.5H4z" />
          </svg>
        </button>
        <button
          className="view-cube-action"
          aria-label="Fit to objects"
          title="Fit to objects"
          disabled={!canFitObjects}
          onClick={() => onFit("objects")}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.3"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M1.5 4.5v-3h3M11.5 1.5h3v3M14.5 11.5v3h-3M4.5 14.5h-3v-3" />
            <path d="M8 4.5l3 1.6v3.6L8 11.5 5 9.7V6.1zM5 6.1l3 1.7 3-1.7M8 7.8v3.7" />
          </svg>
        </button>
      </div>
    </div>
  );
}
