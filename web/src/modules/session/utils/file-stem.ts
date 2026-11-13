/** A file name without its extension, as the CLI names its outputs. */
export function fileStem(name: string): string {
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(0, dot) : name;
}
