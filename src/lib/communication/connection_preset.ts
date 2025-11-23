/** A saved host/port combination for quick switching between environments. */
export interface ConnectionPreset {
  id: string;
  name: string;
  hostname: string;
  port: number;
}

/** Creates a new preset with a unique ID. */
export function createPreset(
  name: string,
  hostname: string,
  port: number,
): ConnectionPreset {
  return {
    id: crypto.randomUUID(),
    name,
    hostname,
    port,
  };
}
