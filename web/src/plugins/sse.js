import { buildAuthHeaders } from "./axios";

export class HeaderEventSource {
  constructor(url, options = {}) {
    this.url = url;
    this.options = options;
    this.listeners = {};
    this.controller = new AbortController();
    this.CONNECTING = 0;
    this.OPEN = 1;
    this.CLOSED = 2;
    this.readyState = this.CONNECTING;
    this._start();
  }

  addEventListener(type, handler) {
    this.listeners[type] = this.listeners[type] || [];
    this.listeners[type].push(handler);
  }

  close() {
    if (this.readyState === this.CLOSED) {
      return;
    }
    this.readyState = this.CLOSED;
    this.controller.abort();
  }

  _emit(type, data = "") {
    const event = { type, data };
    (this.listeners[type] || []).forEach(handler => {
      try {
        handler(event);
      } catch (error) {
        //
      }
    });
  }

  async _start() {
    try {
      const response = await fetch(this.url, {
        method: "GET",
        headers: {
          Accept: "text/event-stream",
          ...buildAuthHeaders(),
          ...(this.options.headers || {})
        },
        credentials: this.options.withCredentials ? "include" : "same-origin",
        signal: this.controller.signal
      });

      if (!response.ok || !response.body) {
        const text = await response.text().catch(() => "");
        this.readyState = this.CLOSED;
        this._emit("error", text);
        return;
      }

      this.readyState = this.OPEN;

      const reader = response.body.getReader();
      const decoder = new TextDecoder("utf-8");
      let buffer = "";
      let eventName = "message";
      let eventData = [];

      const flushEvent = () => {
        if (!eventData.length) {
          eventName = "message";
          return;
        }
        this._emit(eventName || "message", eventData.join("\n"));
        eventName = "message";
        eventData = [];
      };

      while (this.readyState !== this.CLOSED) {
        const { done, value } = await reader.read();
        if (done) {
          break;
        }
        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split(/\r?\n/);
        buffer = lines.pop() || "";

        lines.forEach(line => {
          if (!line) {
            flushEvent();
            return;
          }
          if (line.startsWith(":")) {
            return;
          }
          const idx = line.indexOf(":");
          const field = idx >= 0 ? line.slice(0, idx) : line;
          let valueText = idx >= 0 ? line.slice(idx + 1) : "";
          if (valueText.startsWith(" ")) {
            valueText = valueText.slice(1);
          }
          if (field === "event") {
            eventName = valueText || "message";
          } else if (field === "data") {
            eventData.push(valueText);
          }
        });
      }

      if (buffer.trim().startsWith("data:")) {
        eventData.push(
          buffer
            .trim()
            .slice(5)
            .trimStart()
        );
      }
      flushEvent();
      this.readyState = this.CLOSED;
    } catch (error) {
      if (this.readyState !== this.CLOSED) {
        this.readyState = this.CLOSED;
        this._emit("error", error && error.message ? error.message : "");
      }
    }
  }
}
