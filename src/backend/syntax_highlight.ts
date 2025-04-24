import { invoke } from "@tauri-apps/api/core";

export async function syntaxHighlight(message: string): Promise<string> {
  return invoke("syntax_highlight", { message });
}

