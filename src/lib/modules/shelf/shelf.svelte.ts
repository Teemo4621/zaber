import { invoke } from "@tauri-apps/api/core";
import { startDrag } from "@crabnebula/tauri-plugin-drag";
import { persisted } from "$lib/store.svelte";

export type ShelfItem = { path: string; name: string; addedAt: number };

const MAX_ITEMS = 30;
const asItems = (v: unknown): ShelfItem[] => (Array.isArray(v) ? v.filter((i) => typeof i?.path === "string") : []);
export const shelf = persisted<ShelfItem[]>("shelf", [], asItems);

export const shelfUi = $state({ dragging: false });

export function addFiles(paths: string[]) {
  const fresh = paths
    .filter((p) => !shelf.value.some((i) => i.path === p))
    .map((path) => ({ path, name: path.split(/[\\/]/).pop() || path, addedAt: Date.now() }));
  if (fresh.length) shelf.save([...fresh, ...shelf.value].slice(0, MAX_ITEMS));
}

export const removeFile = (path: string) => shelf.save(shelf.value.filter((i) => i.path !== path));
export const clearShelf = () => shelf.save([]);
export const openFile = (path: string, reveal = false) => invoke("shelf_open", { path, reveal });

export const icons = $state<Record<string, string | null>>({});
const loading = new Set<string>();
export function loadIcon(path: string) {
  if (path in icons || loading.has(path)) return;
  loading.add(path);
  invoke<string | null>("shelf_icon", { path })
    .then((src) => (icons[path] = src))
    .catch(() => {})
    .finally(() => setTimeout(() => loading.delete(path), 1000));
}

export const extension = (name: string) => (name.includes(".") ? name.split(".").pop()!.slice(0, 4).toUpperCase() : "FILE");

let dragIcon: string | undefined;
function icon() {
  if (dragIcon) return dragIcon;
  const c = document.createElement("canvas");
  c.width = 40;
  c.height = 48;
  const g = c.getContext("2d")!;
  g.fillStyle = "#f5f5f7";
  g.beginPath();
  g.roundRect(0, 0, 40, 48, 6);
  g.fill();
  return (dragIcon = c.toDataURL("image/png"));
}

export const dragOut = (path: string) => startDrag({ item: [path], icon: icon() });
