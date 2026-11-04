import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type StoreName = "settings" | "todo" | "water" | "shelf" | "notes";

export function persisted<T>(name: StoreName, initial: T, merge: (saved: unknown) => T = (s) => (s ?? initial) as T) {
  const box = $state({ value: initial, loaded: false });
  invoke<unknown>("store_get", { name })
    .then((saved) => {
      box.value = merge(saved);
    })
    .finally(() => {
      box.loaded = true;
    });
  listen<{ name: string; value: unknown }>("store:update", (e) => {
    if (e.payload.name === name) box.value = merge(e.payload.value);
  });
  return {
    get value() {
      return box.value;
    },
    get loaded() {
      return box.loaded;
    },
    save(next: T) {
      box.value = next;
      return invoke("store_set", { name, value: $state.snapshot(next) });
    },
  };
}
