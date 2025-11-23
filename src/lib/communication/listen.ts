/**
 * Bridge module for running an MLLP listen server to receive incoming HL7 messages.
 *
 * Provides a way to start/stop a local MLLP server that receives HL7 messages from
 * other systems. This is used during development and testing to simulate receiving
 * messages from the database or other HL7 systems.
 *
 * ## Listen Server Flow
 *
 * 1. User clicks "Listen" in the UI, triggering `startListening()`
 * 2. Frontend sets up event listener for "received-message" events
 * 3. Frontend invokes Tauri command "start_listening" with host and port
 * 4. Rust backend:
 *    - Starts a Tokio async TCP listener on the specified host:port
 *    - Stores the server's JoinHandle in app state
 *    - For each incoming connection:
 *      - Reads MLLP-framed message
 *      - Emits "received-message" event with the message text
 *      - Sends ACK response back to sender
 * 5. Frontend receives "received-message" events and adds to Svelte store
 * 6. UI displays received messages with unread indicators
 * 7. When user clicks "Stop", `stopListening()` cancels the server task
 *
 * ## Why Svelte Store Integration?
 *
 * The `messages` store is passed directly to the event handler so that received
 * messages are immediately reactive in the UI. This allows the message list and
 * unread count to update automatically without manual state management.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Writable } from "svelte/store";

/**
 * Sets up a listener for incoming HL7 messages and adds them to the messages store.
 *
 * This should be called once when the app starts, before `startListening()` is called,
 * to ensure no messages are missed. The listener remains active even when the server
 * is stopped, ready to receive messages when the server is restarted.
 *
 * @param messages - Svelte writable store that will be updated with received messages
 * @returns Function to call to stop listening (should be called on app unmount)
 */
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

/**
 * Starts the MLLP listen server on the specified host and port.
 *
 * The listening store is set to false initially to handle the case where starting
 * the server fails (e.g., port already in use). Only after the server successfully
 * starts is the store set to true, ensuring the UI accurately reflects server state.
 *
 * @param host - Hostname/IP to bind to (null means all interfaces: 0.0.0.0)
 * @param port - Port number to listen on (typically 2575 for HL7)
 * @param listening - Svelte writable store tracking whether server is running
 * @throws Error if server fails to start (port in use, permission denied, etc.)
 */
export async function startListening(
  host: string | null,
  port: number,
  listening: Writable<boolean>,
): Promise<void> {
  host = host || null;
  console.info("startListening", host, port);
  // Set to false first in case start fails
  listening.set(false);
  await invoke("start_listening", {
    host,
    port,
  });
  // Only set to true after successful start
  listening.set(true);
}

/**
 * Stops the MLLP listen server.
 *
 * The backend cancels the server task's JoinHandle, which closes all active
 * connections and stops accepting new connections. The listening store is
 * updated to reflect that the server is no longer running.
 *
 * @param listening - Svelte writable store tracking whether server is running
 */
export async function stopListening(
  listening: Writable<boolean>,
): Promise<void> {
  console.info("stopListening");
  await invoke("stop_listening");
  listening.set(false);
}
