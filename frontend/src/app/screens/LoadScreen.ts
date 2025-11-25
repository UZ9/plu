import { CircularProgressBar } from "@pixi/ui";
import { Container } from "pixi.js";

/** Screen shown while loading assets */
export class LoadScreen extends Container {
  /** Assets bundles required by this screen */
  public static assetBundles = ["preload"];
  /** Progress Bar */
  private progressBar: CircularProgressBar;

  constructor() {
    super();

    this.progressBar = new CircularProgressBar({
      backgroundColor: "#3d3d3d",
      fillColor: "#e79e64",
      radius: 100,
      lineWidth: 15,
      value: 20,
      backgroundAlpha: 0.5,
      fillAlpha: 0.8,
      cap: "round",
    });

    this.progressBar.x += this.progressBar.width / 2;
    this.progressBar.y += -this.progressBar.height / 2;

    this.addChild(this.progressBar);
  }

  public onLoad(progress: number) {
    this.progressBar.progress = progress;
  }

  /** Resize the screen, fired whenever window size changes  */
  public resize(width: number, height: number) {
    this.progressBar.position.set(width * 0.5, height * 0.5);
  }
}
