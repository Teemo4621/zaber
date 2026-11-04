export function vividColor(px: ArrayLike<number>): string | null {
  let [r, g, b, total] = [0, 0, 0, 0];
  let [ar, ag, ab] = [0, 0, 0];
  for (let i = 0; i < px.length; i += 4) {
    const max = Math.max(px[i], px[i + 1], px[i + 2]);
    const min = Math.min(px[i], px[i + 1], px[i + 2]);
    [ar, ag, ab] = [ar + px[i], ag + px[i + 1], ab + px[i + 2]];
    if (max < 40) continue;
    const sat = (max - min) / max;
    const w = sat * sat * (max / 255);
    [r, g, b, total] = [r + px[i] * w, g + px[i + 1] * w, b + px[i + 2] * w, total + w];
  }
  if (px.length === 0) return null;
  if (total < 0.02 * (px.length / 4)) {
    const n = px.length / 4;
    [r, g, b] = [ar / n, ag / n, ab / n];
  } else [r, g, b] = [r / total, g / total, b / total];
  const lift = 235 / Math.max(r, g, b, 1);
  return `rgb(${[r, g, b].map((v) => Math.min(255, Math.round(v * lift))).join(",")})`;
}
