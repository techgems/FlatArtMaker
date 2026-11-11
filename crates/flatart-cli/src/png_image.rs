//! Reading a PNG into an `RgbaImage`.

use flatart_domain::{Error, Result, Rgba, RgbaImage};

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::InvalidInput(format!("can't read the PNG: {error}"))
}

/// Any PNG (grey, RGB, palette, with or without alpha, 8 or 16 bits) as
/// 8-bit RGBA.
pub fn read_png(bytes: &[u8]) -> Result<RgbaImage> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(invalid)?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| invalid("the image is too big"))?;
    let mut buffer = vec![0; size];
    let info = reader.next_frame(&mut buffer).map_err(invalid)?;
    let data = &buffer[..info.buffer_size()];
    let pixel = |c: &[u8]| match *c {
        [g] => Rgba {
            r: g,
            g,
            b: g,
            a: 255,
        },
        [g, a] => Rgba { r: g, g, b: g, a },
        [r, g, b] => Rgba { r, g, b, a: 255 },
        [r, g, b, a, ..] => Rgba { r, g, b, a },
        _ => Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        },
    };
    let channels = info.color_type.samples();
    let pixels = data.chunks_exact(channels).map(pixel).collect();
    RgbaImage::new(info.width, info.height, pixels)
}
