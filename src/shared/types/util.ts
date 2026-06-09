export type Writable<T> = { -readonly [K in keyof T]: T[K] };

export type Numberish = string | number;
