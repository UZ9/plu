import * as PIXI from "pixi.js";
import { HexTile } from "../../../types/types";
import { hexPoints, hexToPixel } from "../utils/mathUtils";

const terrainColors: Record<string, number> = {
  Mine: 0x2196f3,
  Slime: 0x6c2d47,
  Wild: 0x4caf50,
  Turret: 0x2196f3,
};

export class HexagonView extends PIXI.Graphics {
  public data: HexTile;

  constructor(data: HexTile, row: number, col: number) {
    super();
    this.data = data;

    const q = col - Math.floor(row / 2);
    const r = row;

    const { x, y } = hexToPixel(q, r);

    this.x = x;
    this.y = y;

    this.interactive = true;
    this.cursor = "pointer";
  }

  public draw() {
    let color = 0xff0000;

    if (typeof this.data === "string") {
      color = terrainColors[this.data as string];
    } else {
      color = terrainColors[Object.keys(this.data)[0]];
    }

    this.clear();
    this.poly(hexPoints).stroke({ width: 2, color: 0x2b2b2b }).fill(color);
  }
}
