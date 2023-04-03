import { Buffer } from "buffer";
import { create } from "zustand";
import { calculateMemoryHash } from "./Utils";

globalThis.Buffer = Buffer;

export const useStore = create((set, get) => ({
  status: null,
  program: [],
  memory: new Uint8Array(64 * 1024),

  setStatus: (status) => set({ status }),
  setProgram: (program) => set({ program }),
  setMemory: (memory) => set({ memory }),

  fetchStatus: async () => {
    const response = await fetch("/api/status");
    const status = await response.json();
    set({ status });
  },

  fetchProgram: async () => {
    const response = await fetch("/api/program");
    const program = await response.json();
    set({ program });
  },

  fetchMemory: async () => {
    const memory = get().memory;
    const memoryHash = calculateMemoryHash(memory);
    const response = await fetch(`/api/memory?hash=${memoryHash}`);
    const deltaMemory = await response.json();

    Object.entries(deltaMemory).forEach(([addr, value]) => {
      memory[addr] = value;
    });

    set({ memory });
  },

  step: async () => {
    const response = await fetch("/api/step", { method: "POST" });
    const status = await response.json();
    set({ status });
  },

  reset: async () => {
    const response = await fetch("/api/reset", { method: "POST" });
    const status = await response.json();
    set({ status });
  },
}));
