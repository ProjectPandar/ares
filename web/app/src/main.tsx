import { RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { router } from "./router";
import "./styles.css";

if (import.meta.env.VITE_ARES_E2E === "1") void import("./e2eHooks").then(({ installE2eHooks }) => installE2eHooks());

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);
