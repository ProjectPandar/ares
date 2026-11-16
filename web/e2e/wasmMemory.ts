import type { Browser } from "@playwright/test";

type Reply = { id: number; result?: unknown; error?: unknown };
type MemoryRecord = { targetId: string; url: string; linearBytes: number[] };

/** Inspect the unchanged worker's grow-only linear memory, not JS heap or process RSS. */
export async function wasmMemory(browser: Browser, origin: string): Promise<MemoryRecord[]> {
  const cdp = await browser.newBrowserCDPSession();
  const records: MemoryRecord[] = [];
  try {
    const { targetInfos } = await cdp.send("Target.getTargets");
    for (const target of targetInfos.filter(({ type, url }) => type === "worker" && url.startsWith(origin))) {
      const { sessionId } = await cdp.send("Target.attachToTarget", { targetId: target.targetId, flatten: false });
      let next = 0;
      const pending = new Map<number, { resolve: (value: unknown) => void; reject: (error: Error) => void }>();
      const receive = (event: { sessionId: string; message: string }) => {
        if (event.sessionId !== sessionId) return;
        const message: Reply = JSON.parse(event.message);
        const entry = pending.get(message.id);
        if (!entry) return;
        pending.delete(message.id);
        if (message.error) entry.reject(new Error(JSON.stringify(message.error)));
        else entry.resolve(message.result);
      };
      cdp.on("Target.receivedMessageFromTarget", receive);
      const request = <T>(method: string, params: object = {}) => new Promise<T>((resolve, reject) => {
        const id = ++next;
        pending.set(id, { resolve: (value) => resolve(value as T), reject });
        cdp.send("Target.sendMessageToTarget", { sessionId, message: JSON.stringify({ id, method, params }) }).catch(reject);
      });
      try {
        const prototype = await request<{ result: { objectId: string } }>("Runtime.evaluate", {
          expression: "WebAssembly.Memory.prototype", objectGroup: "memory-measurement",
        });
        const objects = await request<{ objects: { objectId: string } }>("Runtime.queryObjects", {
          prototypeObjectId: prototype.result.objectId, objectGroup: "memory-measurement",
        });
        const sizes = await request<{ result: { value: number[] } }>("Runtime.callFunctionOn", {
          objectId: objects.objects.objectId,
          functionDeclaration: "function() { return Array.from(this, memory => memory.buffer.byteLength); }",
          returnByValue: true,
        });
        records.push({ targetId: target.targetId, url: target.url, linearBytes: sizes.result.value });
        await request("Runtime.releaseObjectGroup", { objectGroup: "memory-measurement" });
      } finally {
        cdp.off("Target.receivedMessageFromTarget", receive);
        await cdp.send("Target.detachFromTarget", { sessionId });
      }
    }
    return records;
  } finally {
    await cdp.detach();
  }
}
