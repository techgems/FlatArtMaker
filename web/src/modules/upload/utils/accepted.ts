/** The image types the app reads (SPEC "Scope"). */
export const ACCEPTED_TYPES = ['image/png', 'image/jpeg', 'image/webp'] as const;

export function isAcceptedImage(file: File): boolean {
  return (ACCEPTED_TYPES as readonly string[]).includes(file.type);
}
