/** Hands `bytes` to the browser as a download named `fileName`. */
export function saveFile(bytes: Uint8Array, fileName: string, mimeType: string): void {
  // Copied so the bytes sit in a plain ArrayBuffer, which Blob requires.
  const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: mimeType }));
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.click();
  // Some browsers start the download after click() returns; revoking at once can cancel it.
  setTimeout(() => URL.revokeObjectURL(url), 0);
}
