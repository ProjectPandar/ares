import {
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  redirect,
} from "@tanstack/react-router";
import { Shell } from "./shell/Shell";

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

export const router = createRouter({
  routeTree: rootRoute.addChildren([indexRoute, prepareRoute, previewRoute]),
  history: createHashHistory(),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
