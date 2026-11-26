import { useRouter, useRouterState } from "@tanstack/react-router";
import { type DragEvent, lazy, Suspense, useEffect, useLayoutEffect, useRef, useState } from "react";
import { exportGcode, importFiles, selectObject, slicePlate } from "../state/actions";
import { PreparePage } from "../prepare/PreparePage";
import { PreviewPage } from "../preview/PreviewPage";
import { Sidebar } from "../sidebar/Sidebar";
import { DevicesPage } from "../devices/DevicesPage";
import { useApp } from "../state/store";
import { usePlacedScene } from "../viewport/usePlacedScene";
import { AboutDialog } from "./AboutDialog";
import { cmdDown, isShortcut } from "./keys";
import { LoadWarning } from "./LoadWarning";
import { MultipartDialog } from "./MultipartDialog";
import { CloudSignInDialog, PrintHostSendDialog } from "./PrintHostDialogs";
import { EngineCrashDialog } from "./EngineCrashDialog";
import { ShortcutsDialog } from "./ShortcutsDialog";
import { StatusBar } from "./StatusBar";
import { TabBar, TitleBar } from "./TopBar";

const SetupWizard = lazy(() => import("../wizard/SetupWizard").then((module) => ({ default: module.SetupWizard })));

const hasFiles = (event: DragEvent) => event.dataTransfer.types.includes("Files");

export function Shell() {
  const theme = useApp((state) => state.theme);
  const engine = useApp((state) => state.engine);
  const wizard = useApp((state) => state.wizard);
  const stlImport = useApp((state) => state.stlImport);
  const [dragging, setDragging] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [cloudDialog, setCloudDialog] = useState<"sign-in" | "sign-in-to-send" | "send" | null>(null);
  const importInput = useRef<HTMLInputElement>(null);
  const projectInput = useRef<HTMLInputElement>(null);
  const router = useRouter();
  // One Sidebar and both tab panes, built once; the route shows one (Plater::priv::set_current_panel,
  // Plater.cpp:12564-12865).
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const preview = pathname === "/preview";
  const signedIn = useApp((state) => state.cloud !== null);
  const placed = usePlacedScene();
  // Layout effect: CSS theme variables must be live before the viewport reads them.
  useLayoutEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  // GUI_App::request_user_logout drops the Device tab (MainFrame::show_device(false)): leave it first.
  useEffect(() => {
    if (!signedIn && pathname === "/devices") void router.navigate({ to: "/prepare" });
  }, [signedIn, pathname, router]);

  // MainFrame key handler (Ctrl+O/I/R/G) and plater keys: ? list, Esc deselect, Tab switches tabs.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!isShortcut(event) || event.altKey) return;
      const { engine: status, loadingFile, scene, slice } = useApp.getState();
      if (cmdDown(event)) {
        const key = event.key.toLowerCase();
        if (event.shiftKey || !["o", "i", "r", "g"].includes(key)) return;
        event.preventDefault();
        if (key === "o" && status === "ready" && !loadingFile && slice.status !== "slicing")
          projectInput.current!.click();
        else if (key === "i" && status === "ready" && !loadingFile) importInput.current!.click();
        else if (key === "r" && status === "ready" && scene) {
          void slicePlate();
          void router.navigate({ to: "/preview" });
        } else if (key === "g") exportGcode();
      } else if (event.key === "?") setShortcutsOpen(true);
      else if (event.key === "Escape") selectObject(null);
      // GLCanvas3D::on_key: plain Tab on the canvas, i.e. while no control has focus.
      else if (event.key === "Tab" && !event.shiftKey && event.target === document.body) {
        event.preventDefault();
        void router.navigate({ to: router.state.location.pathname === "/preview" ? "/prepare" : "/preview" });
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [router]);

  return (
    <div className="shell">
      <TitleBar
        importInput={importInput}
        projectInput={projectInput}
        onShortcuts={() => setShortcutsOpen(true)}
        onAbout={() => setAboutOpen(true)}
        onSignIn={() => setCloudDialog("sign-in")}
      />
      <TabBar onPrint={() => setCloudDialog(useApp.getState().cloud ? "send" : "sign-in-to-send")} />
      <main
        className="workspace"
        data-testid="workspace"
        onDragOver={(event) => {
          if (!hasFiles(event) || engine !== "ready") return;
          event.preventDefault();
          setDragging(true);
        }}
        onDragLeave={(event) => {
          if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setDragging(false);
        }}
        onDrop={(event) => {
          if (!hasFiles(event) || engine !== "ready") return;
          event.preventDefault();
          setDragging(false);
          importFiles([...event.dataTransfer.files]);
        }}
      >
        <div className="page" hidden={pathname === "/devices"}>
          <Sidebar />
          <PreparePage shown={!preview} placed={placed} />
          <PreviewPage shown={preview} placed={placed} />
        </div>
        {pathname === "/devices" && <DevicesPage />}
        <LoadWarning />
        {dragging && (
          <div className="drop-overlay" data-testid="drop-overlay">
            Drop a 3MF project or STL model to import
          </div>
        )}
      </main>
      <StatusBar />
      {stlImport && (
        // A replaced dialog's late close event must not clear the import that replaced it.
        <MultipartDialog
          key={stlImport.id}
          files={stlImport.files}
          onClose={() => useApp.setState((state) => (state.stlImport?.id === stlImport.id ? { stlImport: null } : {}))}
        />
      )}
      <ShortcutsDialog open={shortcutsOpen} onClose={() => setShortcutsOpen(false)} />
      <EngineCrashDialog />
      <AboutDialog open={aboutOpen} onClose={() => setAboutOpen(false)} />
      {(cloudDialog === "sign-in" || cloudDialog === "sign-in-to-send") && (
        <CloudSignInDialog
          onClose={() => setCloudDialog((open) => (open === "send" ? open : null))}
          onSignedIn={() => setCloudDialog(cloudDialog === "sign-in-to-send" ? "send" : null)}
        />
      )}
      {cloudDialog === "send" && signedIn && (
        <PrintHostSendDialog onClose={() => setCloudDialog((open) => (open === "send" ? null : open))} />
      )}
      {wizard && (
        <Suspense>
          <SetupWizard
            key={wizard}
            firstRun={wizard === "first-run"}
            onClose={() => useApp.setState({ wizard: null })}
          />
        </Suspense>
      )}
    </div>
  );
}
