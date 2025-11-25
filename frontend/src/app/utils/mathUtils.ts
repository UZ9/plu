export const HEX_SIZE = 40;

const sqrt3 = Math.sqrt(3);

export const hexToPixel = (q: number, r: number) => ({
  x: HEX_SIZE * sqrt3 * (q + r / 2),
  y: HEX_SIZE * 1.5 * r,
});

// precompute hexagon points
export const hexPoints: number[] = [];

for (let i = 0; i < 6; i++) {
  const angle = (Math.PI / 180) * (30 + i * 60);
  hexPoints.push(
    Math.cos(angle) * HEX_SIZE,
    Math.sin(angle) * HEX_SIZE
  );
}
