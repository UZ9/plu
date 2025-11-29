import * as PIXI from "pixi.js";
import { HexTile } from "../../../../types/types";
import { WebSocketManager } from "../../network/socket";
import { HexagonView } from "../../components/HexagonView";

const TOOLTIP_STYLE: PIXI.TextStyleOptions = {
  fontFamily: "monospace",
  fontSize: 16,
  fill: 0xffffff,
  stroke: { color: 0x000000, width: 3 },
}

const DEFAULT_TERRAIN: HexTile = "Slime";

export class HexMapScreen extends PIXI.Container {
  private tooltip!: PIXI.Text;
  private tooltipBackground!: PIXI.Graphics;
  private hexes: HexagonView[] = [];
  // TODO: switch hexMap over to array
  private mapContainer: PIXI.Container;
  private initialized: boolean = false;

  private wsClient: WebSocketManager;

  private isDragging = false;
  private dragStart = { x: 0, y: 0 };
  private mapStart = { x: 0, y: 0 };

  private map_width = 0;

  constructor() {
    super();
    this.interactive = true;

    this.mapContainer = new PIXI.Container();
    this.addChild(this.mapContainer);

    this.initTooltip();
    this.registerMapContainerEvents();

    this.wsClient = new WebSocketManager(this.handleWebSocketMessage.bind(this));
    this.wsClient.connect();
  }

  private initTooltip() {
      this.tooltipBackground = new PIXI.Graphics();
      this.tooltipBackground.visible = false;
      this.addChild(this.tooltipBackground);

      this.tooltip = new PIXI.Text({ text: "", style: TOOLTIP_STYLE });

      this.tooltip.visible = false;
      this.addChild(this.tooltip);
  }

  private registerMapContainerEvents() {
    this.mapContainer.interactive = true;
    this.mapContainer.on("pointerdown", this.onDragStart.bind(this));
    this.mapContainer.on("pointerup", this.onDragEnd.bind(this));
    this.mapContainer.on("pointerupoutside", this.onDragEnd.bind(this));
    this.mapContainer.on("pointermove", this.onDragMove.bind(this));
  }

  private handleWebSocketMessage(message: any) {
    switch (message.type) {
      case 'grid_state':
        console.log("grid_state")
        if (!this.initialized) {
      console.log({message});
        this.createHexGrid(message.width, message.height);
        this.map_width = message.width;
      }

      this.updateGridState(message.tiles);
      break;
      case 'tile_update':
        this.updateTile({col: message.col, data: message.data, row: message.row});
      break;
      case 'tiles_update':
        message.tiles.forEach((tile: any) => this.updateTile(tile));
      break;
      default:
        console.warn('Unknown message type:', message.type);
    }
  }

  private updateGridState(tiles: Array<{ col: number; row: number; data: Partial<HexTile> }>) {
    console.log("updatign");
    tiles.forEach(tile => {
      this.updateTile(tile);
    });
  }

  private updateTile({col, row, data}: { col: number; row: number; data: Partial<HexTile> }) {

    const index = this.map_width * row + col;

    const hex = this.hexes[index];

    if (hex) {
      // rust type gen compatibility is quite terrible, it can be either string or object for enum
      // also js strings are primitives, not objects? :(
      if (typeof data === 'string') {
        hex.data = data;
      } else {
        Object.assign(hex.data, data);
      }

      if (data) {
        hex.draw();
      }
    }

    console.log("done");
  }

  public sendTileUpdate(col: number, row: number, data: Partial<HexTile>) {
    this.wsClient.sendMessage(JSON.stringify({
      type: 'tile_update',
      col,
      row,
      data
    }));
  }

  private onDragStart(event: PIXI.FederatedPointerEvent) {
    this.isDragging = true;
    const pos = event.global;

    this.dragStart.x = pos.x;
    this.dragStart.y = pos.y;
    this.mapStart.x = this.mapContainer.x;
    this.mapStart.y = this.mapContainer.y;
  }

  private onDragEnd() {
    this.isDragging = false;
  }

  private onDragMove(event: PIXI.FederatedPointerEvent) {
    if (!this.isDragging) return;

    const pos = event.global;
    const dx = pos.x - this.dragStart.x;
    const dy = pos.y - this.dragStart.y;

    this.mapContainer.x = this.mapStart.x + dx;
    this.mapContainer.y = this.mapStart.y + dy;
  }

  private registerHexEvent(g: HexagonView, row: number, col: number) {
    g.on("pointerout", () => {
      this.tooltip.visible = false;
      this.tooltipBackground.visible = false;
    });

    g.on("pointerover", (_) => {
      if (!this.isDragging) {
        this.tooltip.text = `x: ${col}, y: ${row}, terrain: ${g.data}`;
        this.tooltip.visible = true;
        this.tooltipBackground.visible = true;
      }
    })

    g.on("pointerdown", (ev) => {
      if (ev.button === 0 && !this.isDragging) {
        const newTerrain: HexTile = "Slime";

        this.sendTileUpdate(col, row, newTerrain);

        console.log(`Clicked hex at [${col},${row}]:`, g.data);
      }
    });

    g.on("pointermove", (ev) => {
      if (!this.isDragging) {
        const pos = ev.global;
        this.tooltip.position.set(pos.x + 10, pos.y - 10);

        const padding = 5;
        const bounds = this.tooltip.getBounds();

        this.tooltipBackground.clear();

        this.tooltipBackground.rect(
          bounds.x - padding,
          bounds.y - padding,
          bounds.width + padding * 2,
          bounds.height + padding * 2,
        )
        .stroke({ width: 1, color: 0xffffff, alpha: 0.5 })
        .fill({ color: 0x000000, alpha: 0.8})
      }
    });
  }

  private createHexGrid(width: number, height: number) {
    this.initialized = true;

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const g = new HexagonView(DEFAULT_TERRAIN, row, col);

        g.draw();

        this.registerHexEvent(g, row, col);

        this.mapContainer.addChild(g);
        this.hexes.push(g);
      }
    }
  }

  public resize() {
    this.mapContainer.x = 0;
    this.mapContainer.y = 0;
  }
}
