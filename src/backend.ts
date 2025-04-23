import { invoke } from "@tauri-apps/api/core";

export async function syntaxHighlight(message: string): Promise<string> {
  return invoke("syntax_highlight", { message });
}

export interface LocatedCursor {
  segment: string | null;
  field: number | null;
  repeat: number | null;
  component: number | null;
  subcomponent: number | null;
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
