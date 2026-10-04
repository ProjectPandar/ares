import type { ReactNode, SVGProps } from "react";

function Icon({ children, ...props }: SVGProps<SVGSVGElement> & { children: ReactNode }) {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor"
      strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" aria-hidden {...props}>
      {children}
    </svg>
  );
}

export const ImportIcon = () => (
  <Icon><path d="M2.5 10.5v3h11v-3M8 2.5v8M5 7.5l3 3 3-3" /></Icon>
);
export const ExportIcon = () => (
  <Icon><path d="M2.5 10.5v3h11v-3M8 10.5v-8M5 5.5l3-3 3 3" /></Icon>
);
export const SunIcon = () => (
  <Icon><circle cx="8" cy="8" r="3" /><path d="M8 1v1.5M8 13.5V15M1 8h1.5M13.5 8H15M3 3l1 1M12 12l1 1M3 13l1-1M12 4l1-1" /></Icon>
);
export const MoonIcon = () => (
  <Icon><path d="M13.5 9.5A5.5 5.5 0 0 1 6.5 2.5a5.5 5.5 0 1 0 7 7z" /></Icon>
);
export const PrinterIcon = () => (
  <Icon><rect x="2" y="2" width="12" height="12" rx="1" /><path d="M2 5h12M6 5v3h4V5M4 12h8" /></Icon>
);
export const FilamentIcon = () => (
  <Icon><circle cx="8" cy="8" r="5.5" /><circle cx="8" cy="8" r="1.8" /></Icon>
);
export const ProcessIcon = () => (
  <Icon><path d="M2 4h7M12 4h2M2 8h2M7 8h7M2 12h9M14 12h0" /><circle cx="10.5" cy="4" r="1.5" /><circle cx="5.5" cy="8" r="1.5" /><circle cx="12.5" cy="12" r="1.5" /></Icon>
);
export const ObjectIcon = () => (
  <Icon><path d="M8 1.5l5.5 3v7L8 14.5l-5.5-3v-7zM2.5 4.5L8 7.5l5.5-3M8 7.5v7" /></Icon>
);
export const CubeSmallIcon = () => (
  <Icon width="12" height="12"><path d="M8 1.5l5.5 3v7L8 14.5l-5.5-3v-7zM2.5 4.5L8 7.5l5.5-3M8 7.5v7" /></Icon>
);
export const SliceIcon = () => (
  <Icon><path d="M2 4h12M2 7h12M2 10h12M2 13h12" /></Icon>
);
