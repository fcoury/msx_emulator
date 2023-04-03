import { create } from "zustand";

export const useStore = create((set) => ({
  status: null,
  program: [],
  memory: [],

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
    const response = await fetch("/api/memory");
    const memory = await response.json();
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
