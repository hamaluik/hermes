import { invoke } from "@tauri-apps/api/core";

export async function loadSpec(
  segment: string,
  field: number | null,
  component: number | null,
): Promise<string | null> {
  return invoke("get_std_description", { segment, field, component });
}

export async function loadDescription(
  segment: string,
  field: number | null,
  component: number | null,
): Promise<string | null> {
  return invoke("get_wizard_description", { segment, field, component });
}

export async function getSpecAndDescription(
  segment: string,
  field: number | null,
  component: number | null,
): Promise<{ spec: string | null; description: string | null }> {
  return Promise.all([
    loadSpec(segment, field, component),
    loadDescription(segment, field, component),
  ]).then(([specResult, descriptionResult]) => {
    return { spec: specResult, description: descriptionResult };
  });
}
