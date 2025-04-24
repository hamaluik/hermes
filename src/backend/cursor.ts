import { invoke } from "@tauri-apps/api/core";

export interface LocatedCursor {
  segment?: string;
  field?: number;
  repeat?: number;
  component?: number;
  subcomponent?: number;
}

export async function locateCursor(
  message: string,
  cursor: number,
): Promise<LocatedCursor | null> {
  return invoke("locate_cursor", {
    message,
    cursor,
  });
}
