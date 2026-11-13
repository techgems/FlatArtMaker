import type { Rgb } from '../../engine';

/** Black or white, whichever reads better on `background` (WCAG relative luminance). */
export function textOn(background: Rgb): '#000' | '#fff' {
  const linear = (channel: number): number => {
    const c = channel / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  const luminance =
    0.2126 * linear(background.r) + 0.7152 * linear(background.g) + 0.0722 * linear(background.b);
  return luminance > 0.179 ? '#000' : '#fff';
}
