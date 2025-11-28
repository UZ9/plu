import * as PIXI from "pixi.js";
import { HexData, TerrainType } from "../../../types/types";
import { hexPoints, hexToPixel } from "../utils/mathUtils";

const terrainColors: Record<TerrainType, number> = {
  "Mine": 0x4CAF50,
  "Slime": 0x757575,
  "Wild": 0x4CAF50,
  "Turret": 0x2196F3,
};

export class HexagonView extends PIXI.Graphics {
  public data: HexData;

  constructor(data: HexData, row: number, col: number) {
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
    const color = terrainColors[this.data.terrain];

    this.clear();
    this.poly(hexPoints).stroke({ width: 2, color: 0x2b2b2b }).fill(color);
  }
}
