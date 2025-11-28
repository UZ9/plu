// socket logic for connecting to game backend

import { logDebug, logError, logInfo } from "../utils/logger";

const BACKEND_URL="ws://localhost:9001/ws"

export type WebSocketMessageHandler = (message: any) => void;

export class WebSocketManager {
  private socket?: WebSocket;
  private handler: WebSocketMessageHandler;

  constructor(handler: WebSocketMessageHandler) {
    this.handler = handler;
  }

  public connect() {
    this.socket = new WebSocket(BACKEND_URL);
    this.registerEvents();
  }

  private handleWebSocketMessage(message: any) {
    this.handler(message);
  }

  private registerEvents() {
    if (!this.socket) {
      return;
    }

    this.socket.onopen = () => {
      logInfo("WebSocket connected")
      this.socket?.send(JSON.stringify({ type: 'request_grid_state' }));
    };

    this.socket.onmessage = (event) => {
      try {
        logDebug(`socket message: ${event}`);
        const message = JSON.parse(event.data);
        this.handleWebSocketMessage(message);
      } catch (error) {
        logError(`Failed to parse WebSocket message: ${error}`);
      }
    };

    this.socket.onerror = (error) => {
      logError(`WebSocket error: ${error}`);
    };

    this.socket.onclose = () => {
      logInfo("WebSocket closed");
      setTimeout(() => this.connect(), 3065);
    };
  }

  public sendMessage(message: string) {
    if (this.socket && this.socket.readyState == WebSocket.OPEN) {
      this.socket.send(message);
    }
  }
}
