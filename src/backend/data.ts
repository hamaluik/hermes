import { invoke } from "@tauri-apps/api/core";
import type { SegmentSchema } from "./schema";

export interface SegmentData {
  /// fields accessed by their "<segment>.<field>(.<component>)?" name
  /// i.e. "MSH.1" or "MSH.1.2"
  fields: Record<string, string | null>;
}

export async function getMessageSegmentNames(
  message: string,
): Promise<string[]> {
  try {
    return await invoke("get_message_segment_names", { message });
  } catch (error) {
    console.error("Error getting message segment names:", error);
    throw error;
  }
}

export async function getMessageTriggerEvent(
  message: string,
): Promise<string | null> {
  try {
    return await invoke("get_message_trigger_event", { message });
  } catch (error) {
    console.error("Error getting message trigger event:", error);
    throw error;
  }
}

export async function parseMessageSegment(
  message: string,
  segment: string,
  segmentRepeat: number,
): Promise<SegmentData> {
  try {
    return await invoke("parse_message_segment", {
      message,
      segment,
      segmentRepeat,
    });
  } catch (error) {
    console.error("Error parsing message segment:", error);
    throw error;
  }
}

export async function renderMessageSegment(
  message: string,
  segment: string,
  segmentRepeat: number,
  data: SegmentData,
): Promise<string> {
  return await invoke("render_message_segment", {
    message,
    segment,
    segmentRepeat,
    data,
  });
}

export function generateDefaultData(
  segment: string,
  schema: SegmentSchema,
): SegmentData {
  const data: SegmentData = { fields: {} };
  for (const field of schema) {
    if (field.group) continue;
    const fieldName = `${segment}.${field.field}`;
    if (field.component) {
      data.fields[`${fieldName}.${field.component}`] = null;
    } else {
      data.fields[fieldName] = null;
    }
  }
  return data;
}
