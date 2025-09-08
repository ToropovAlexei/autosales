export interface IFilter {
  [key: string]:
    | string
    | number
    | string[]
    | number[]
    | boolean
    | undefined
    | null;
}

export type FalsyValues = false | null | 0 | "" | undefined;
