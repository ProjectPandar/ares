import type { ToolMarker } from "@ares-slicer/wasm";

const formats = new Map<number, Intl.NumberFormat>();

/** printf `%.Nf`: half to even on the exact float value, as glibc rounds. */
export function printf(value: number, decimals: number) {
  let format = formats.get(decimals);
  if (!format) {
    format = new Intl.NumberFormat("en-US", {
      minimumFractionDigits: decimals, maximumFractionDigits: decimals, roundingMode: "halfEven", useGrouping: false,
    });
    formats.set(decimals, format);
  }
  return format.format(value);
}

/** `get_time_dhms` (Utils.hpp:540-562): display-only f32 decomposition. */
export function timeDhms(timeInSecs: number) {
  const f32 = Math.fround;
  let seconds = f32(timeInSecs);
  const days = Math.trunc(f32(seconds / 86400));
  seconds = f32(seconds - f32(f32(days) * 86400));
  const hours = Math.trunc(f32(seconds / 3600));
  seconds = f32(seconds - f32(f32(hours) * 3600));
  const minutes = Math.trunc(f32(seconds / 60));
  seconds = f32(seconds - f32(f32(minutes) * 60));
  const whole = Math.trunc(seconds);
  return days > 0 ? `${days}d ${hours}h ${minutes}m ${whole}s`
    : hours > 0 ? `${hours}h ${minutes}m ${whole}s`
      : minutes > 0 ? `${minutes}m ${whole}s`
        : seconds > 1 ? `${whole}s` : `${printf(seconds, 6)}s`;
}

/** GCodeViewer.cpp:452-454. */
export function formatMarkerTime({ elapsedSeconds, vertexSeconds }: ToolMarker["time"]) {
  return `${timeDhms(elapsedSeconds)} (${printf(Math.fround(vertexSeconds), 3)}s)`;
}

/** `short_time` (Utils.hpp:494-537): rereads a `timeDhms` string, display only. */
export function shortTime(time: string) {
  const ints = () => (time.match(/-?\d+/g) ?? []).map(Number);
  let [days, hours, minutes, seconds, fSeconds] = [0, 0, 0, 0, 0];
  if (time.includes("d")) [days, hours, minutes, seconds] = ints();
  else if (time.includes("h")) [hours, minutes, seconds] = ints();
  else if (time.includes("m")) [minutes, seconds] = ints();
  else if (time.includes("s")) {
    // sscanf "%fs" into a float.
    fSeconds = Math.fround(parseFloat(time));
    seconds = Math.trunc(fSeconds);
  }
  if (days + hours > 0 && seconds >= 30 && ++minutes === 60) {
    minutes = 0;
    if (++hours === 24) {
      hours = 0;
      ++days;
    }
  }
  return days > 0 ? `${days}d${hours}h${minutes}m`
    : hours > 0 ? `${hours}h${minutes}m`
      : minutes > 0 ? `${minutes}m${seconds}s`
        : seconds >= 1 ? `${seconds}s`
          : fSeconds > 0 && fSeconds < 1 ? "<1s" : "0s";
}

/** `format_compact_weight` (GCodeViewer.cpp:128-150), metric display. */
export function compactWeight(grams: number) {
  const abs = Math.abs(grams);
  const [value, unit] = abs >= 1e6 ? [abs / 1e6, "t"] : abs >= 1000 ? [abs / 1000, "kg"] : [abs, "g"];
  return `${grams < 0 ? "-" : ""}${printf(value, 2)}${unit}`;
}
