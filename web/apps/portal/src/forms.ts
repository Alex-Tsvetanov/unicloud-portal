/** Reads a trimmed string field from a form. */
export function str(fd: FormData, name: string): string {
  const v = fd.get(name);
  return typeof v === "string" ? v.trim() : "";
}

/** Reads an integer field. Returns NaN when it is empty or not a whole number. */
export function int(fd: FormData, name: string): number {
  const s = str(fd, name);
  return /^-?\d+$/.test(s) ? Number(s) : Number.NaN;
}

/** Splits a textarea into non-empty lines. */
export function lines(fd: FormData, name: string): string[] {
  return str(fd, name)
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean);
}
