import { invoke } from "@tauri-apps/api/core";
import {
  listen,
  type Event as ListenEvent,
  type UnlistenFn,
} from "@tauri-apps/api/event";

export interface SendRequest {
  host: string;
  port: number;
  wait_timeout_seconds: number;
  message: string;
}

export async function sendMessage(
  request: SendRequest,
  onSendLog?: (log: string) => void,
): Promise<string | null> {
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
    unlistenResponse?.();
  });

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
    unlistenResponse?.();
    unlistenLog?.();
  }

  return Promise.all([responsePromise, logPromise]).then(
    ([response]) => response,
  );
}

export type SendResponse =
  | { event: "failedToConnect"; data: string }
  | { event: "failedToSend"; data: string }
  | { event: "failedToReceive"; data: string }
  | { event: "failedToDecode"; data: string }
  | { event: "failedToParse"; data: { message: string; error: string } }
  | { event: "final"; data: string | null };

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
