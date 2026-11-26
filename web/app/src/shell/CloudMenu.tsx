import { useEffect, useRef, useState } from "react";
import { signOut } from "../state/cloud";
import { useApp } from "../state/store";

/**
 * The top bar's Pandar account menu: sign in (GUI_App::request_login, GUI_App.cpp:5363) or, once
 * signed in, the account and sign out (GUI_App::request_user_logout, :5448). Closes on a pick,
 * Escape or a click elsewhere.
 */
export function CloudMenu({ onSignIn }: { onSignIn: () => void }) {
  const cloud = useApp((state) => state.cloud);
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (event: Event) => {
      if (event instanceof KeyboardEvent ? event.key === "Escape" : !root.current?.contains(event.target as Node))
        setOpen(false);
    };
    document.addEventListener("pointerdown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("pointerdown", close);
      document.removeEventListener("keydown", close);
    };
  }, [open]);
  const pick = (action: () => void) => () => {
    setOpen(false);
    action();
  };
  return (
    <div ref={root} className="cloud-menu">
      <button className="bar-button" aria-haspopup="menu" aria-expanded={open} onClick={() => setOpen(!open)}>
        Cloud
      </button>
      {open && (
        <div className="context-menu" role="menu" aria-label="Cloud">
          {cloud ? (
            <>
              <span className="context-menu-label">
                {cloud.userName} · {cloud.tenantName}
              </span>
              <button type="button" role="menuitem" onClick={pick(signOut)}>
                Sign out
              </button>
            </>
          ) : (
            <button type="button" role="menuitem" onClick={pick(onSignIn)}>
              Sign in…
            </button>
          )}
        </div>
      )}
    </div>
  );
}
