import { describe, expect, it, vi } from 'vitest';
import { UploadZone } from './upload-zone.svelte';

const png = new File([new Uint8Array([1])], 'art.png', { type: 'image/png' });
const pdf = new File([new Uint8Array([1])], 'art.pdf', { type: 'application/pdf' });

describe('UploadZone', () => {
  it('passes on the first image', () => {
    const onfile = vi.fn();
    new UploadZone(onfile).take([png, pdf]);
    expect(onfile).toHaveBeenCalledWith(png);
  });

  it('refuses a file that isn’t PNG, JPG or WebP, and says why', () => {
    const onfile = vi.fn();
    const zone = new UploadZone(onfile);
    zone.take([pdf]);
    expect(onfile).not.toHaveBeenCalled();
    expect(zone.rejected).toMatch(/art\.pdf/);
    zone.take([png]);
    expect(zone.rejected).toBeNull();
  });
});
