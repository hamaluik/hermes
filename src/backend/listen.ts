import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Writable } from "svelte/store";

export async function listenToListenResponse(
  messages: Writable<{ message: string; unread: boolean }[]>,
): Promise<UnlistenFn> {
  console.log("listenToListenResponse");
  return listen<string>("received-message", (event) => {
    console.log("received-message", event);
    if (event.payload) {
      messages.update((currentMessages) => {
        const newMessage = {
          message: event.payload,
          unread: true,
        };
        return [...currentMessages, newMessage];
      });
    }
  });
}

export async function startListening(
  host: string | null,
  port: number,
  listening: Writable<boolean>,
): Promise<void> {
  host = host || null;
  console.info("startListening", host, port);
  listening.set(false);
  await invoke("start_listening", {
    host,
    port,
  });
  listening.set(true);
}

export async function stopListening(
  listening: Writable<boolean>,
): Promise<void> {
  console.info("stopListening");
  await invoke("stop_listening");
  listening.set(false);
}
