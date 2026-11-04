import { test } from "node:test";
import assert from "node:assert/strict";
import { vividColor } from "./color.ts";

const image = (...colors) => colors.flatMap(([r, g, b]) => [r, g, b, 255]);

test("gray artwork keeps its own neutral tone, lifted", () => {
  assert.equal(vividColor(image([128, 128, 128], [40, 40, 40], [220, 220, 220])), "rgb(235,235,235)");
  assert.equal(vividColor([]), null);
});

test("a little color wins over a lot of gray, lifted to read on black", () => {
  const px = image(...Array(10).fill([120, 120, 120]), [200, 40, 40], [180, 30, 30]);
  const [r, g, b] = vividColor(px).match(/\d+/g).map(Number);
  assert.equal(r, 235);
  assert.ok(g < 60 && b < 60, `still red, got ${g},${b}`);
});
