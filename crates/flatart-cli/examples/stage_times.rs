//! Times each stage on an image, with the defaults:
//! `cargo run --release -p flatart-cli --example stage_times -- in.png`
use flatart_cli::{Job, read_png};
use std::time::Instant;

fn main() {
    let path = std::env::args().nth(1).expect("an image path");
    let image = read_png(&std::fs::read(path).unwrap()).unwrap();
    let mut p = Job::new(image).pipeline().unwrap();
    let t = Instant::now();
    p.loaded().unwrap();
    eprintln!("load {:?}", t.elapsed());
    let t = Instant::now();
    let n = p.detection().unwrap().palette.colours().len();
    eprintln!("detect {:?} ({n} colours)", t.elapsed());
    let t = Instant::now();
    p.mapped().unwrap();
    eprintln!("map {:?}", t.elapsed());
    let t = Instant::now();
    p.cleaned().unwrap();
    eprintln!("clean {:?}", t.elapsed());
    let t = Instant::now();
    p.printable().unwrap();
    eprintln!("printable {:?}", t.elapsed());
    let t = Instant::now();
    let b = p.border_map().unwrap();
    eprintln!(
        "trace {:?} ({} borders, {} faces)",
        t.elapsed(),
        b.borders.len(),
        b.faces.len()
    );
    let t = Instant::now();
    let m = p.mesh().unwrap();
    let tris: usize = m.parts.iter().map(|x| x.mesh.triangles.len()).sum();
    eprintln!(
        "mesh {:?} ({} parts, {tris} triangles)",
        t.elapsed(),
        m.parts.len()
    );
}
