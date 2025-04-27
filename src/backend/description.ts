import { invoke } from "@tauri-apps/api/core";

export async function loadSpec(
  segment: string,
  field: number | null,
  component: number | null,
): Promise<string | null> {
  return invoke("get_std_description", { segment, field, component });
}
