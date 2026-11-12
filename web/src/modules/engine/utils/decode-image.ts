import type { RgbaPixels } from '../types';

/**
 * Decodes an image file in the browser, which keeps decoders out of the WASM
 * file. Straight alpha and no colour conversion, so the pixels are the file's.
 */
export async function decodeImage(file: Blob): Promise<RgbaPixels> {
  const bitmap = await createImageBitmap(file, {
    premultiplyAlpha: 'none',
    colorSpaceConversion: 'none',
  });
  try {
    const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
    const context = canvas.getContext('2d');
    if (context === null) throw new Error('This browser has no 2D canvas');
    context.drawImage(bitmap, 0, 0);
    const { data, width, height } = context.getImageData(0, 0, bitmap.width, bitmap.height);
    return { width, height, data: new Uint8Array(data.buffer, data.byteOffset, data.byteLength) };
  } finally {
    bitmap.close();
  }
}
