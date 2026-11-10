//! Small inputs shared by the suites and by adapter tests.

use flatart_domain::settings::{
    Advanced, CornerThreshold, Extrusion, FitSettings, LayerHeight, MappingSettings,
    OutputColourTarget, PrintScale, SmoothingTolerance, TraceSettings, TraceUpscale,
};
use flatart_domain::{
    Border, BorderEnds, BorderId, BorderMap, ColourName, CubicBezier, Degrees, DetectedColour,
    DetectedPalette, Detection, Face, Label, LabelMap, Loop, LoopStep, Mesh, MeshModel, MeshPart,
    Mm, MmPerPx, MmPoint3, Oklab, OutputColour, OutputColourId, OutputPalette, PartKind, Px,
    PxPoint, Rgb, Rgba, RgbaImage, VectorModel,
};

pub const RED: Rgba = Rgba {
    r: 220,
    g: 30,
    b: 30,
    a: 255,
};
pub const BLUE: Rgba = Rgba {
    r: 30,
    g: 60,
    b: 220,
    a: 255,
};
const CLEAR: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

/// An image from ASCII rows: `.` is transparent, `r` red, `b` blue.
pub fn image(rows: &[&str]) -> RgbaImage {
    let height = rows.len() as u32;
    let width = rows.first().map_or(0, |r| r.len()) as u32;
    let pixels = rows
        .iter()
        .flat_map(|row| row.chars())
        .map(|c| match c {
            'r' => RED,
            'b' => BLUE,
            _ => CLEAR,
        })
        .collect();
    RgbaImage::new(width, height, pixels).expect("fixture image rows must be equal length")
}

/// A label map from ASCII rows: `.` is outside, a digit is that label.
pub fn labels(rows: &[&str], upscale: u8) -> LabelMap {
    let height = rows.len() as u32;
    let width = rows.first().map_or(0, |r| r.len()) as u32;
    let cells = rows
        .iter()
        .flat_map(|row| row.chars())
        .map(|c| c.to_digit(10).map_or(Label::OUTSIDE, |d| Label(d as u16)))
        .collect();
    let upscale = TraceUpscale::new(upscale).expect("fixture upscale must be 1 to 4");
    LabelMap::new(width, height, upscale, cells).expect("fixture rows must be equal length")
}

/// A red square with a blue centre, inside a transparent margin.
pub fn two_colour_image() -> RgbaImage {
    image(&[
        "......", //
        ".rrrr.", //
        ".rbbr.", //
        ".rbbr.", //
        ".rrrr.", //
        "......",
    ])
}

/// Label 0 with an island of label 1: two closed borders, no junctions.
pub fn nested_labels() -> LabelMap {
    labels(
        &[
            "......", //
            ".0000.", //
            ".0110.", //
            ".0110.", //
            ".0000.", //
            "......",
        ],
        1,
    )
}

/// Labels 0 and 1 side by side: junctions where they meet the outside.
pub fn split_labels() -> LabelMap {
    labels(
        &[
            "......", //
            ".0011.", //
            ".0011.", //
            ".0011.", //
            "......",
        ],
        1,
    )
}

/// Three labels meeting at a point, touching the image edge.
pub fn three_labels() -> LabelMap {
    labels(
        &[
            "0011", //
            "0011", //
            "2222", //
        ],
        1,
    )
}

/// Diagonal neighbours of the same label: four faces under 4-connectivity.
pub fn diagonal_labels() -> LabelMap {
    labels(
        &[
            "01", //
            "10", //
        ],
        1,
    )
}

/// Every label-map fixture, named.
pub fn all_label_maps() -> Vec<(&'static str, LabelMap)> {
    vec![
        ("nested", nested_labels()),
        ("split", split_labels()),
        ("three", three_labels()),
        ("diagonal", diagonal_labels()),
        ("nested ×2", upscaled(&nested_labels(), 2)),
    ]
}

/// The same map with each cell repeated `k` × `k` times.
pub fn upscaled(map: &LabelMap, k: u8) -> LabelMap {
    let (w, h, k32) = (map.width(), map.height(), u32::from(k));
    let mut cells = Vec::with_capacity((w * h * k32 * k32) as usize);
    for y in 0..h * k32 {
        for x in 0..w * k32 {
            cells.push(map.get(x / k32, y / k32).unwrap_or(Label::OUTSIDE));
        }
    }
    let upscale = TraceUpscale::new(k).expect("upscale must be 1 to 4");
    LabelMap::new(w * k32, h * k32, upscale, cells).expect("sizes match by construction")
}

/// The default fit settings: 0.5 px tolerance, 60° corners.
pub fn fit_settings() -> FitSettings {
    let advanced = Advanced::default();
    FitSettings {
        tolerance: advanced.smoothing,
        corner: advanced.corner,
    }
}

/// Fit settings with an explicit tolerance and corner angle.
pub fn fit_settings_with(tolerance: f64, corner: f64) -> FitSettings {
    FitSettings {
        tolerance: SmoothingTolerance::new(Px(tolerance)).expect("tolerance 0.1 to 2"),
        corner: CornerThreshold::new(Degrees(corner)).expect("corner 0 to 180"),
    }
}

pub fn trace_settings() -> TraceSettings {
    TraceSettings {
        fit: fit_settings(),
    }
}

/// Four detected colours on a 4 × 4 map: black (label 0), a near-black
/// 2 ΔE away (1), white (2) and red (3). Black and near-black cover most of
/// the outer edge.
///
/// ```text
/// 0001
/// 0012
/// 2233
/// .333
/// ```
pub fn detection() -> Detection {
    let colour = |l, a, b, pixels| DetectedColour {
        colour: Oklab::new(l, a, b).expect("finite fixture colour"),
        pixels,
    };
    Detection {
        palette: DetectedPalette::new(vec![
            colour(0.20, 0.0, 0.0, 5),
            colour(0.22, 0.0, 0.0, 2),
            colour(0.95, 0.0, 0.0, 3),
            colour(0.63, 0.22, 0.13, 5),
        ])
        .expect("valid fixture palette"),
        labels: labels(&["0001", "0012", "2233", ".333"], 1),
    }
}

/// Mapping settings asking for `target` output colours.
pub fn mapping_settings(target: u8) -> MappingSettings {
    MappingSettings {
        target: OutputColourTarget::new(target).expect("target 1 to 16"),
    }
}

/// `Black` (slot 1, also the base colour) and `White` (slot 2).
pub fn palette() -> OutputPalette {
    let colour = |id: u16, name: &str, v: u8| OutputColour {
        id: OutputColourId(id),
        name: ColourName::new(name).expect("valid fixture name"),
        display: Rgb { r: v, g: v, b: v },
    };
    OutputPalette::new(
        vec![colour(0, "Black", 20), colour(1, "White", 240)],
        Some(OutputColourId(0)),
    )
    .expect("valid fixture palette")
}

fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<CubicBezier> {
    let p = PxPoint::new;
    let corners = [p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)];
    (0..4)
        .map(|i| CubicBezier::line(corners[i], corners[(i + 1) % 4]))
        .collect()
}

/// The border map of [`nested_labels`], built by hand: label 0's square ring
/// around label 1's 2 × 2 island.
pub fn nested_border_map() -> BorderMap {
    let step = |border: u32, reversed: bool| LoopStep {
        border: BorderId(border),
        reversed,
    };
    BorderMap {
        width: Px(6.0),
        height: Px(6.0),
        junctions: vec![],
        borders: vec![
            // Clockwise on screen, so 0 is on the right.
            Border {
                ends: BorderEnds::Closed,
                curve: square(1.0, 1.0, 5.0, 5.0),
                left: Label::OUTSIDE,
                right: Label(0),
            },
            Border {
                ends: BorderEnds::Closed,
                curve: square(2.0, 2.0, 4.0, 4.0),
                left: Label(0),
                right: Label(1),
            },
        ],
        faces: vec![
            Face {
                colour: OutputColourId(0),
                outer: Loop {
                    steps: vec![step(0, false)],
                },
                holes: vec![Loop {
                    steps: vec![step(1, true)],
                }],
            },
            Face {
                colour: OutputColourId(1),
                outer: Loop {
                    steps: vec![step(1, false)],
                },
                holes: vec![],
            },
        ],
    }
}

/// [`nested_border_map`] at 10 mm per pixel, 2 mm base, 1 mm colour layer.
pub fn vector_model() -> VectorModel {
    VectorModel {
        borders: nested_border_map(),
        palette: palette(),
        scale: PrintScale::new(MmPerPx(10.0)).expect("valid scale"),
        extrusion: Extrusion {
            colour: LayerHeight::new(Mm(1.0)).expect("valid height"),
            base: Some(LayerHeight::new(Mm(2.0)).expect("valid height")),
            fill_base_holes: true,
        },
    }
}

/// A closed box with outward-facing triangles.
pub fn box_mesh(min: [f64; 3], max: [f64; 3]) -> Mesh {
    let vertices = (0..8)
        .map(|i| {
            let pick = |axis: usize| {
                if i >> axis & 1 == 0 {
                    min[axis]
                } else {
                    max[axis]
                }
            };
            MmPoint3::new(pick(0), pick(1), pick(2))
        })
        .collect();
    // Vertex i has x from bit 0, y from bit 1, z from bit 2.
    let triangles = vec![
        [0, 2, 1],
        [1, 2, 3], // bottom (z min), facing down
        [4, 5, 6],
        [5, 7, 6], // top
        [0, 1, 4],
        [1, 5, 4], // y min
        [2, 6, 3],
        [3, 6, 7], // y max
        [0, 4, 2],
        [2, 4, 6], // x min
        [1, 3, 5],
        [3, 7, 5], // x max
    ];
    Mesh {
        vertices,
        triangles,
    }
}

/// A 40 × 40 × 2 mm `Base` (slot 1) and a 20 × 20 × 1 mm `White` square on
/// top (slot 2), as in the task 02 samples.
pub fn mesh_model() -> MeshModel {
    MeshModel {
        palette: palette(),
        parts: vec![
            MeshPart {
                kind: PartKind::Base,
                colour: OutputColourId(0),
                mesh: box_mesh([0.0, 0.0, 0.0], [40.0, 40.0, 2.0]),
            },
            MeshPart {
                kind: PartKind::Colour,
                colour: OutputColourId(1),
                mesh: box_mesh([10.0, 10.0, 2.0], [30.0, 30.0, 3.0]),
            },
        ],
    }
}
