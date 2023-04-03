import XXH from "xxhashjs";

export function calculateMemoryHash(memory) {
  const memoryBuffer = new Uint8Array(
    Object.entries(memory)
      .sort(([a], [b]) => a - b)
      .flatMap(([_, value]) => value)
  );

  const hash = XXH.h64(memoryBuffer, 0);
  return hash.toString(10);
}
