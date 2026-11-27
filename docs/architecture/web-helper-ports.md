# Configurable web test helper ports

This changes Ares test infrastructure only. There is no OrcaSlicer source to port: the browser servers and fake Hub are owned by Ares. Existing slicer comparisons, package checks, and printer-job assertions remain unchanged.

## Loopback helpers

Use one optional environment variable, `ARES_WEB_HELPER_PORT`, to select a TCP port for each of the following helpers. When unset, keep the current port. Bind to `127.0.0.1`, navigate Chromium to the bound address, and do not silently fall back to another port if the selected port is busy.

| Ares source and destination | Default | Current behavior |
| --- | --- | --- |
| [`web/e2e/wasmMoves.mjs:80–87`](../../web/e2e/wasmMoves.mjs#L80-L87) | `4245` | Node binds a fixed port; navigation already uses `server.address().port`. |
| [`web/e2e/package.spec.ts:82–90`](../../web/e2e/package.spec.ts#L82-L90), [`web/e2e/ksrProject.spec.ts:42–52`](../../web/e2e/ksrProject.spec.ts#L42-L52) | `4245` | The packed WASM consumer and KSR page each hardcode both the Vite port and the navigation URL. |
| [`web/e2e/profiles.spec.ts:112–120`](../../web/e2e/profiles.spec.ts#L112-L120) | `4176` | The packed profiles consumer hardcodes both the Vite port and the navigation URL. |
| [`web/e2e/profileSlice.spec.ts:79–87`](../../web/e2e/profileSlice.spec.ts#L79-L87) | `4234` | The profile slicing page also hardcodes both the Vite port and the navigation URL. |

Keep Vite's `strictPort` and existing teardown. Use the resolved Vite URL as in [`configRoundTrip.spec.ts:46–56`](../../web/e2e/configRoundTrip.spec.ts#L46-L56) and [`platerSession.spec.ts:54–62`](../../web/e2e/platerSession.spec.ts#L54-L62); those two already use ephemeral ports and need no change. The [`packages` project](../../web/playwright.config.ts#L35-L40) runs one test at a time, so its helpers can share one override. Concurrent invocations must pick distinct ports.

## Pandar test origin

[`fakeHub`](../../web/e2e/pandarHub.ts#L8-L13) hardcodes the app origin for CORS and [validates the sign-in callback against the same origin](../../web/e2e/pandarHub.ts#L27-L43). Both [`pandarSend.spec.ts:51–57`](../../web/e2e/pandarSend.spec.ts#L51-L57) and [`pandarCloud.spec.ts:50–60`](../../web/e2e/pandarCloud.spec.ts#L50-L60) exercise sign-in through this stub. Pass the Playwright `baseURL` fixture from each caller and use `new URL(baseURL).origin` in `fakeHub` for both the exact `Access-Control-Allow-Origin` value and the exact `/cloud-callback.html` URL. Do not use a wildcard or reflect the request's `Origin`; leave the fake Hub's address, callback validation, and responses otherwise unchanged.

The main Playwright preview remains configured separately by its [`baseURL`](../../web/playwright.config.ts#L19-L22) and [`webServer`](../../web/playwright.config.ts#L59-L63). Do not change production CORS, app dev/preview defaults, or the separate Lighthouse and Pages harnesses.

## Verification

Run the standalone WASM move helper and the four affected package specs once with the helper override unset and once with a private port. Run both Pandar specs with the normal preview and with a private preview whose launch port, readiness URL, and `baseURL` agree. Keep the existing comparisons and assertions in all runs. Run web `fmt:check`, `lint`, `typecheck`, and `test:unit`.
