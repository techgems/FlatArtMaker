//! `flatart`: native entry point. Argument parsing and adapter wiring arrive
//! with work-split task 12 (cli-and-golden-tests).

fn main() {
    let info = flatart_pipeline::engine_info();
    println!("flatart {}", info.version);
}
