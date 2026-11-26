import { createHashHistory, createRootRoute, createRoute, createRouter, redirect } from "@tanstack/react-router";
import { Shell } from "./shell/Shell";
import { useApp } from "./state/store";

const rootRoute = createRootRoute({ component: Shell });

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  beforeLoad: () => {
    throw redirect({ to: "/prepare" });
  },
});

const prepareRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/prepare",
});

const previewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/preview",
});

/** MainFrame's Device tab, only while signed in to Pandar Cloud. */
const devicesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/devices",
  beforeLoad: () => {
    if (!useApp.getState().cloud) throw redirect({ to: "/prepare" });
  },
});

export const router = createRouter({
  routeTree: rootRoute.addChildren([indexRoute, prepareRoute, previewRoute, devicesRoute]),
  history: createHashHistory(),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
