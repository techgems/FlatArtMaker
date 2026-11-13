import { isAcceptedImage } from './utils/accepted';

/** The drop zone's state: whether a file is over it, and why the last one was refused. */
export class UploadZone {
  dragging = $state(false);
  rejected = $state<string | null>(null);

  readonly #onfile: (file: File) => void;

  constructor(onfile: (file: File) => void) {
    this.#onfile = onfile;
  }

  /** Takes the first file, if it's an image the app reads. */
  take(files: FileList | readonly File[] | null | undefined): void {
    this.dragging = false;
    const file = files?.[0];
    if (file === undefined) return;
    if (!isAcceptedImage(file)) {
      this.rejected = `${file.name} isn’t a PNG, JPG or WebP image.`;
      return;
    }
    this.rejected = null;
    this.#onfile(file);
  }
}
