//! The SVG file: a layer per output colour, a path per face.

use crate::path;
use crate::place::{Placement, number};
use flatart_domain::VectorModel;
use std::fmt::Write;

/// Escapes text for an XML attribute or element.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The whole document. Layers are Inkscape layers (`groupmode="layer"`)
/// whose id and label are the output colour's name, in slot order; colours
/// with no faces are left out.
pub fn write(model: &VectorModel, title: &str) -> String {
    let map = &model.borders;
    let place = Placement::new(map, model.scale.get().0);
    let (w, h) = (number(place.width), number(place.height));
    let mut svg = String::new();
    let _ = writeln!(svg, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" width="{w}mm" height="{h}mm" viewBox="0 0 {w} {h}">"#
    );
    if !title.is_empty() {
        let _ = writeln!(svg, "  <title>{}</title>", escape(title));
    }
    for colour in model.palette.colours() {
        let faces: Vec<_> = map.faces.iter().filter(|f| f.colour == colour.id).collect();
        if faces.is_empty() {
            continue;
        }
        let name = escape(colour.name.as_str());
        let fill = format!(
            "#{:02x}{:02x}{:02x}",
            colour.display.r, colour.display.g, colour.display.b
        );
        let _ = writeln!(
            svg,
            r#"  <g id="{name}" inkscape:groupmode="layer" inkscape:label="{name}" fill="{fill}" fill-rule="evenodd">"#
        );
        for face in faces {
            let _ = writeln!(svg, r#"    <path d="{}"/>"#, path::face(map, face, &place));
        }
        let _ = writeln!(svg, "  </g>");
    }
    let _ = writeln!(svg, "</svg>");
    svg
}
