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

export async function getRangeOfPreviousField(
  message: string,
  cursor: number,
): Promise<{ start: number; end: number } | null> {
  return invoke("get_range_of_previous_field", {
    message,
    cursor,
  });
}

export async function getRangeOfNextField(
  message: string,
  cursor: number,
): Promise<{ start: number; end: number } | null> {
  return invoke("get_range_of_next_field", {
    message,
    cursor,
  });
}
