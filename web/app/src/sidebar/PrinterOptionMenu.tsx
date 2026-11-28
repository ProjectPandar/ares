import { Menu } from "@base-ui/react/menu";
import { useId } from "react";

interface PrinterOption {
  value: string;
  label: string;
}

/** coss UI Menu's radio group, styled with the sidebar's existing control tokens. */
export function PrinterOptionMenu({
  label,
  name = label,
  value,
  options,
  onSelect,
}: {
  label: string;
  name?: string;
  value: string;
  options: PrinterOption[];
  onSelect: (value: string) => void;
}) {
  const description = useId();
  return (
    <div className="kv-row">
      <span>{label}</span>
      <Menu.Root modal={false}>
        <Menu.Trigger className="combo printer-select" aria-label={name} aria-describedby={description}>
          <span className="combo-text" id={description}>
            {options.find((option) => option.value === value)?.label}
          </span>
          <svg className="printer-menu-chevron" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
              d="m4 6 4 4 4-4"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </Menu.Trigger>
        <Menu.Portal>
          <Menu.Positioner align="start" sideOffset={4} className="printer-menu-positioner">
            <Menu.Popup className="printer-menu" aria-label={name}>
              <Menu.RadioGroup value={value} onValueChange={(next) => next !== value && onSelect(next)}>
                {options.map((option) => (
                  <Menu.RadioItem key={option.value} className="printer-menu-item" value={option.value} closeOnClick>
                    <Menu.RadioItemIndicator className="printer-menu-indicator" aria-hidden="true">
                      <svg viewBox="0 0 16 16" width="12" height="12" fill="none">
                        <path d="m3 8 3 3 7-7" stroke="currentColor" strokeWidth="1.5" />
                      </svg>
                    </Menu.RadioItemIndicator>
                    {option.label}
                  </Menu.RadioItem>
                ))}
              </Menu.RadioGroup>
            </Menu.Popup>
          </Menu.Positioner>
        </Menu.Portal>
      </Menu.Root>
    </div>
  );
}
