import { invoke } from "@tauri-apps/api/core";

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

// TODO: get this from the backend
export const KnownSegments = ["MSH"] as const;
export type SegmentSchemas = Record<string, SegmentSchema>;

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
  return Promise.all(
    KnownSegments.map(async (segment) => {
      const schema = await getSegmentSchema(segment);
      return { [segment]: schema };
    }),
  ).then((schemas) => Object.assign({}, ...schemas));
}
