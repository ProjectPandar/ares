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

/** Utils.hpp:540-562 and GCodeViewer.cpp:452-454; display-only f32 decomposition. */
export function formatMarkerTime({ elapsedSeconds, vertexSeconds }: { elapsedSeconds: number; vertexSeconds: number }) {
  const f32 = Math.fround;
  let seconds = f32(elapsedSeconds);
  const days = Math.trunc(f32(seconds / 86400));
  seconds = f32(seconds - f32(f32(days) * 86400));
  const hours = Math.trunc(f32(seconds / 3600));
  seconds = f32(seconds - f32(f32(hours) * 3600));
  const minutes = Math.trunc(f32(seconds / 60));
  seconds = f32(seconds - f32(f32(minutes) * 60));
  const whole = Math.trunc(seconds);
  const elapsed = days > 0 ? `${days}d ${hours}h ${minutes}m ${whole}s`
    : hours > 0 ? `${hours}h ${minutes}m ${whole}s`
      : minutes > 0 ? `${minutes}m ${whole}s`
        : seconds > 1 ? `${whole}s` : `${printf(seconds, 6)}s`;
  return `${elapsed} (${printf(f32(vertexSeconds), 3)}s)`;
}
