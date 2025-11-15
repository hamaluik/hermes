/**
 * Bridge module for sending HL7 messages via MLLP and receiving responses.
 *
 * Uses Tauri's event system to handle async communication with the Rust backend,
 * which streams progress updates (logs) and a final response. This event-based
 * approach is necessary because the backend performs long-running network operations
 * and needs to provide real-time feedback during the send/receive cycle.
 *
 * ## Communication Flow
 *
 * 1. Frontend calls `sendMessage()` with host, port, timeout, and HL7 message
 * 2. Function sets up event listeners for "send-response" and "send-log" events
 * 3. Function invokes Tauri command "send_message" to trigger Rust backend
 * 4. Rust backend:
 *    - Connects to remote MLLP server
 *    - Sends the HL7 message with MLLP framing
 *    - Emits "send-log" events for each step (connecting, sending, waiting, etc.)
 *    - Waits for response (with timeout)
 *    - Parses response if received
 *    - Emits "send-response" event with either error or final response
 * 5. Frontend receives response event and resolves/rejects the promise
 * 6. Cleanup: All event listeners are removed to prevent memory leaks
 *
 * ## Why Events Instead of Direct Returns?
 *
 * The backend needs to provide real-time progress updates during the potentially
 * long-running network operation. Events allow the UI to show connection status,
 * sending progress, and waiting state before the final response arrives.
 */

import { invoke } from "@tauri-apps/api/core";
import {
  listen,
  type Event as ListenEvent,
  type UnlistenFn,
} from "@tauri-apps/api/event";

/**
 * Configuration for sending an HL7 message over MLLP.
 */
export interface SendRequest {
  /** Target hostname or IP address */
  host: string;
  /** Target port number */
  port: number;
  /** Maximum seconds to wait for a response before timing out */
  wait_timeout_seconds: number;
  /** Raw HL7 message string to send */
  message: string;
}

/**
 * Sends an HL7 message over MLLP and awaits the response.
 *
 * Sets up event listeners BEFORE invoking the backend command to avoid race conditions
 * where events could be emitted before we're ready to receive them. The backend streams
 * log messages during the send/receive process and emits a final response event when done.
 *
 * @param request - Send configuration including host, port, timeout, and message
 * @param onSendLog - Optional callback for real-time log updates during the operation
 * @returns The response message text, or null if no response was received
 * @throws Error string if the send/receive operation fails at any stage
 *
 * @example
 * ```ts
 * const response = await sendMessage(
 *   { host: "127.0.0.1", port: 2575, wait_timeout_seconds: 5, message: "MSH|..." },
 *   (log) => console.log("Progress:", log)
 * );
 * ```
 */
export async function sendMessage(
  request: SendRequest,
  onSendLog?: (log: string) => void,
): Promise<string | null> {
  // Set up response listener before invoking to prevent race condition
  let unlistenResponse: UnlistenFn | undefined;
  const responsePromise = new Promise<string | null>((resolve, reject) => {
    listen<SendResponse>("send-response", (event) => {
      const responseError = getResponseError(event.payload);
      if (responseError) {
        unlistenResponse?.();
        reject(responseError);
        return;
      }
      if (event.payload.event === "final") {
        unlistenResponse?.();
        resolve(event.payload.data);
      }
    }).then((unlistenFn) => {
      unlistenResponse = unlistenFn;
    });
  }).finally(() => {
    // Cleanup listener even if promise is cancelled
    unlistenResponse?.();
  });

  // Set up log listener before invoking to capture all log messages
  let unlistenLog: UnlistenFn | undefined;
  let logPromise = new Promise<void>((resolve) => {
    listen<string>("send-log", (event) => {
      onSendLog?.(event.payload);
    }).then((unlistenFn) => {
      unlistenLog = unlistenFn;
      resolve();
    });
  }).finally(() => {
    unlistenLog?.();
  });

  try {
    await invoke("send_message", {
      request,
    });
  } finally {
    // Ensure listeners are cleaned up even if invoke fails
    unlistenResponse?.();
    unlistenLog?.();
  }

  return Promise.all([responsePromise, logPromise]).then(
    ([response]) => response,
  );
}

/**
 * Discriminated union of all possible response events from the backend.
 *
 * The backend emits different event types to indicate failures at various stages
 * of the send/receive process. The "final" event indicates success and contains
 * the response message (or null if no response was received but the operation
 * completed successfully).
 */
export type SendResponse =
  | { event: "failedToConnect"; data: string }
  | { event: "failedToSend"; data: string }
  | { event: "failedToReceive"; data: string }
  | { event: "failedToDecode"; data: string }
  | { event: "failedToParse"; data: { message: string; error: string } }
  | { event: "final"; data: string | null };

/**
 * Extracts a human-readable error message from a SendResponse event.
 *
 * Returns null for the "final" event (success case), allowing callers to easily
 * distinguish between error and success responses.
 *
 * @param response - The response event from the backend
 * @returns Error message string if this is an error event, null otherwise
 */
export function getResponseError(response: SendResponse): string | null {
  if (response.event === "failedToConnect") {
    return `Failed to connect to target: ${response.data}`;
  }
  if (response.event === "failedToSend") {
    return `Failed to send message: ${response.data}`;
  }
  if (response.event === "failedToReceive") {
    return `Failed to receive message: ${response.data}`;
  }
  if (response.event === "failedToDecode") {
    return `Failed to decode message: ${response.data}`;
  }
  if (response.event === "failedToParse") {
    return `Failed to parse message: ${response.data.error})\n\nMessage:\n${response.data.message}`;
  }
  return null;
}

/**
 * Listens to the "send-response" event and invokes the provided handler.
 *
 * Note: The returned function must be called to stop listening to the event.
 * */
export async function listenToSendResponse(
  handler: (event: ListenEvent<SendResponse>) => void,
): Promise<UnlistenFn> {
  return listen<SendResponse>("send-response", handler);
}

/**
 * Listens to the "send-log" event and invokes the provided handler.
 *
 * Note: The returned function must be called to stop listening to the event.
 */
export async function listenToSendLog(
  handler: (event: ListenEvent<string>) => void,
): Promise<UnlistenFn> {
  return listen<string>("send-log", handler);
}
