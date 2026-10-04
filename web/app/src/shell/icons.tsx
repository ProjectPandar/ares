import type { ReactNode, SVGProps } from "react";

// OrcaSlicer resources/images, copied by scripts/extract-process-layout.mjs.
const ORCA_ICONS = Object.fromEntries(
  Object.entries(import.meta.glob<string>("../assets/orca/*.svg", { eager: true, query: "?url", import: "default" }))
    .map(([path, url]) => [path.slice(path.lastIndexOf("/") + 1, -4), url]),
);

export function OrcaIcon({ name, size = 16 }: { name: string; size?: number }) {
  return <img className="orca-icon" src={ORCA_ICONS[name]} width={size} height={size} alt="" draggable={false} />;
}

function Icon({ children, ...props }: SVGProps<SVGSVGElement> & { children: ReactNode }) {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor"
      strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" aria-hidden {...props}>
      {children}
    </svg>
  );
}

export const SunIcon = () => (
  <Icon><circle cx="8" cy="8" r="3" /><path d="M8 1v1.5M8 13.5V15M1 8h1.5M13.5 8H15M3 3l1 1M12 12l1 1M3 13l1-1M12 4l1-1" /></Icon>
);
export const MoonIcon = () => (
  <Icon><path d="M13.5 9.5A5.5 5.5 0 0 1 6.5 2.5a5.5 5.5 0 1 0 7 7z" /></Icon>
);
export const ObjectIcon = () => (
  <Icon><path d="M8 1.5l5.5 3v7L8 14.5l-5.5-3v-7zM2.5 4.5L8 7.5l5.5-3M8 7.5v7" /></Icon>
);
export const CubeSmallIcon = () => (
  <Icon width="12" height="12"><path d="M8 1.5l5.5 3v7L8 14.5l-5.5-3v-7zM2.5 4.5L8 7.5l5.5-3M8 7.5v7" /></Icon>
);
