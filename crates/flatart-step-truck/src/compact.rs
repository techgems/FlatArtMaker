//! Shrinks truck's STEP text without changing what it describes.
//!
//! truck writes a fresh `CARTESIAN_POINT` (or `DIRECTION`) for every use,
//! so a curve's control points appear once for its edge and again for the
//! wall under it. Geometry may be shared in STEP, so each distinct point
//! and direction is kept once and every reference points at it. Topology
//! (vertices, edges) is never merged: coincident vertices that the solids
//! keep apart stay apart.

use std::collections::HashMap;

/// The entities that are merged when their text is identical.
const SHARED: [&str; 2] = ["CARTESIAN_POINT(", "DIRECTION("];

/// One `#id = body;` from the DATA section.
struct Entity<'a> {
    id: u64,
    body: &'a str,
}

/// Splits the DATA section into entities. Bodies run to the first `;`
/// outside a string.
fn entities(data: &str) -> Option<Vec<Entity<'_>>> {
    let mut out = Vec::new();
    let mut rest = data.trim_start();
    while !rest.is_empty() {
        let after_hash = rest.strip_prefix('#')?;
        let digits = after_hash.find(|c: char| !c.is_ascii_digit())?;
        let id = after_hash[..digits].parse().ok()?;
        let body_start = after_hash[digits..].find('=')? + digits + 1;
        let body = &after_hash[body_start..];
        let mut quoted = false;
        let end = body.char_indices().find_map(|(i, c)| {
            match c {
                '\'' => quoted = !quoted,
                ';' if !quoted => return Some(i),
                _ => {}
            }
            None
        })?;
        out.push(Entity {
            id,
            body: body[..end].trim(),
        });
        rest = body[end + 1..].trim_start();
    }
    Some(out)
}

/// Rewrites every `#id` in `body` through `map`.
fn remap(body: &str, map: &HashMap<u64, u64>, out: &mut String) {
    let mut rest = body;
    while let Some(at) = rest.find('#') {
        out.push_str(&rest[..=at]);
        let tail = &rest[at + 1..];
        let digits = tail
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(tail.len());
        match tail[..digits].parse::<u64>() {
            Ok(id) => out.push_str(&map.get(&id).copied().unwrap_or(id).to_string()),
            Err(_) => out.push_str(&tail[..digits]),
        }
        rest = &tail[digits..];
    }
    out.push_str(rest);
}

/// The same file with repeated points and directions written once. Returns
/// the input unchanged if it can't be read as expected.
pub fn compact(step: &str) -> String {
    let Some((head, rest)) = step.split_once("DATA;") else {
        return step.to_owned();
    };
    let Some((data, tail)) = rest.rsplit_once("ENDSEC;") else {
        return step.to_owned();
    };
    let Some(all) = entities(data) else {
        return step.to_owned();
    };
    let mut first: HashMap<&str, u64> = HashMap::new();
    let mut map: HashMap<u64, u64> = HashMap::new();
    for e in &all {
        if SHARED.iter().any(|p| e.body.starts_with(p)) {
            let keep = *first.entry(e.body).or_insert(e.id);
            if keep != e.id {
                map.insert(e.id, keep);
            }
        }
    }
    let mut out = String::with_capacity(step.len());
    out.push_str(head);
    out.push_str("DATA;\n");
    for e in all.iter().filter(|e| !map.contains_key(&e.id)) {
        out.push('#');
        out.push_str(&e.id.to_string());
        out.push_str(" = ");
        remap(e.body, &map, &mut out);
        out.push_str(";\n");
    }
    out.push_str("ENDSEC;");
    out.push_str(tail);
    out
}

#[cfg(test)]
mod tests {
    use super::compact;

    #[test]
    fn repeated_points_are_written_once_and_referenced() {
        let step = "ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\n\
            #1 = CARTESIAN_POINT('', (1.0, 2.0, 0.0));\n\
            #2 = CARTESIAN_POINT('', (1.0, 2.0, 0.0));\n\
            #3 = VERTEX_POINT('', #1);\n\
            #4 = VERTEX_POINT('', #2);\n\
            #5 = LINE('a;b', #2, #6);\n\
            #6 = VECTOR('', #7, 1.0);\n\
            #7 = DIRECTION('', (1.0, 0.0, 0.0));\n\
            ENDSEC;\nEND-ISO-10303-21;\n";
        let out = compact(step);
        assert!(!out.contains("#2 ="), "{out}");
        // Vertices stay two, both on the one point.
        assert!(out.contains("#3 = VERTEX_POINT('', #1);"), "{out}");
        assert!(out.contains("#4 = VERTEX_POINT('', #1);"), "{out}");
        assert!(out.contains("#5 = LINE('a;b', #1, #6);"), "{out}");
        assert!(out.ends_with("ENDSEC;\nEND-ISO-10303-21;\n"), "{out}");
    }
}
