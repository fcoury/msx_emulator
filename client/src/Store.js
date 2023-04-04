import { Buffer } from "buffer";
import { create } from "zustand";
import { calculateMemoryHash } from "./Utils";

globalThis.Buffer = Buffer;

export const useStore = create((set, get) => {
  const handleMessage = (message) => {
    console.log("WebSocket message:", message);
    if (message.type === "status") {
      set({ status: message.data });
    } else if (message.type === "program") {
      set({ program: message.data });
    } else if (message.type === "memory") {
      const memory = get().memory;
      const deltaMemory = message.data;

      Object.entries(deltaMemory).forEach(([addr, value]) => {
        memory[addr] = value;
      });

      set({ memory });
    } else if (message.type === "vram") {
      set({ vram: message.data });
    } else {
      console.error("Unknown WebSocket message type:", message.type);
    }
  };

  const sendMessage = (message, data) => {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: message, data }));
    }
  };

  const ws = new WebSocket("ws://" + window.location.host + "/ws");

  ws.onopen = () => {
    console.log("WebSocket connection opened");
  };

  ws.onmessage = (event) => {
    handleMessage(JSON.parse(event.data));
  };

  ws.onclose = () => {
    console.log("WebSocket connection closed");
  };

  return {
    status: null,
    program: [],
    vram: [],
    memory: new Uint8Array(64 * 1024),
    memoryLoading: false,
    memoryError: null,

    setStatus: (status) => set({ status }),
    setProgram: (program) => set({ program }),
    setMemory: (memory) => set({ memory }),

    fetchStatus: async () => {
      sendMessage("status");
    },

    fetchProgram: async () => {
      sendMessage("program");
    },

    fetchMemory: async () => {
      const hash = calculateMemoryHash(get().memory);
      sendMessage("memory", { hash });
    },

    fetchVram: async () => {
      sendMessage("vram");
    },

    step: async () => {
      sendMessage("step");
    },

    reset: async () => {
      const response = await fetch("/api/reset", { method: "POST" });
      const status = await response.json();
      set({ status });
    },
  };
});
