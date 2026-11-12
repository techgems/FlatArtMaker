# Work notes

Measurements and findings that later tasks may need.

## WASM size (`flatart-wasm`, release)

| Date | Task | `.wasm` | gzip -9 | Vite build (gzip) | Notes |
|---|---|---|---|---|---|
| 2026-11-11 | 14 | 707,917 B | 299,555 B | 707.92 kB (304.29 kB) | All adapters: k-means, mapping, raster, graph tracer, kurbo fitter, mesh, i_triangle, SVG, 3MF (zip + zlib-rs + png). `opt-level = "s"`, LTO, wasm-opt. |

## Browser vs CLI output (task 14)

- The Jolteon 3MF from `npm run dev` (debug button) is byte-identical to the CLI's, for both slicers. 2026-11-11 SHA-256:
  - prusa: 2,021,910 B, `549f3408e4ae7cbe982e6ba8f43ab3ec8ecbfaa85667fb9219b29cdf870ebc1a`
  - bambu: 1,980,681 B, `5ef3fdeed2399dc855bfce38e17df0f773ad628f513b333bde90e897d38d4756`
- This needed the zip host system pinned to Unix in `flatart-3mf`. The `zip` crate records DOS on Windows and Unix elsewhere (WASM included), which changed 5 bytes of the central directory.
- Browser decoding (`createImageBitmap` with `premultiplyAlpha: 'none'`, then a 2D canvas) gave the same pixels as the CLI's PNG decoder for Jolteon.
- In the dev build (debug WASM), the export takes about 17 s with the browser pane hidden. The release build is much faster: about 1.4 s in Node.
