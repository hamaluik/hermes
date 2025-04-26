import { invoke } from "@tauri-apps/api/core";

export type SegmentPaths = Record<string, string>;

export interface SegmentMetadata {
  name: string;
  required?: boolean;
}

export interface MessagesSchema {
  segments: SegmentPaths;
  message: Record<string, SegmentMetadata[]>;
}

export enum DataType {
  Date = "date",
  DateTime = "datetime",
}

export interface Field {
  field: number;
  name: string;
  component?: number;
  group?: string;
  minlength?: number;
  maxlength?: number;
  placeholder?: string;
  required?: boolean;
  datatype?: DataType;
  pattern?: string;
  note?: string;
  values?: Record<string, string>;
}

export type SegmentSchema = Field[];

export type SegmentSchemas = Record<string, SegmentSchema>;

export async function getMessagesSchema(): Promise<MessagesSchema> {
  try {
    return await invoke("get_messages_schema");
  } catch (error) {
    console.error("Error getting messages schema:", error);
    throw error;
  }
}

export async function getSegmentSchema(
  segment: string,
): Promise<SegmentSchema> {
  try {
    return invoke("get_segment_schema", { segment });
  } catch (error) {
    console.error(`Error getting segment {segment} schema:`, error);
    throw error;
  }
}

export async function getAllSegmentSchemas(): Promise<SegmentSchemas> {
  return getMessagesSchema().then(async (schema) => {
    const segments = Object.keys(schema.segments);
    console.debug("Segments to fetch:", segments);
    const schemas = await Promise.all(segments.map(getSegmentSchema));
    console.debug("All segment schemas:", schemas);
    return schemas.reduce((acc, schema, index) => {
      const segment = segments[index];
      acc[segment] = schema;
      return acc;
    }, {} as SegmentSchemas);
  });
}
