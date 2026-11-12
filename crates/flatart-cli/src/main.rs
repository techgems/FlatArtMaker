//! `flatart`: turns a flat-colour PNG into a colour-separated 3MF (and,
//! optionally, an SVG).

use clap::{Parser, ValueEnum};
use flatart_cli::{Fitter, Job, Slicer, ThinFix, read_png};
use flatart_domain::{EditLog, ExportOptions};
use flatart_pipeline::THUMBNAIL_SIDE;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SlicerArg {
    /// PrusaSlicer.
    Prusa,
    /// Bambu Studio and Orca-based slicers.
    Bambu,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FitterArg {
    /// kurbo.
    Kurbo,
    /// visioncortex, the curve fitting behind vtracer.
    Visioncortex,
}

/// Turns a flat-colour PNG into a colour-separated 3MF.
#[derive(Debug, Parser)]
#[command(name = "flatart", version)]
struct Args {
    /// The input PNG.
    input: PathBuf,
    /// The 3MF to write.
    output: PathBuf,
    /// Which slicer the 3MF is for.
    #[arg(long, value_enum)]
    slicer: SlicerArg,
    /// Also write the art as an SVG here.
    #[arg(long)]
    svg: Option<PathBuf>,
    /// Also write the model as a STEP file here, for CAD.
    #[arg(long)]
    step: Option<PathBuf>,
    /// How many output colours (filaments) to start with, 1–16.
    #[arg(long = "output-colors", default_value_t = 8)]
    output_colours: u8,
    /// Base height B in mm. 0 means no base.
    #[arg(long, default_value_t = 3.4)]
    base: f64,
    /// Colour layer height C in mm.
    #[arg(long, default_value_t = 1.0)]
    layer: f64,
    /// The art's width in mm. The height follows the aspect ratio.
    #[arg(long = "size-x", default_value_t = 100.0)]
    size_x: f64,
    /// Turn the thin-feature fix off.
    #[arg(long = "no-thin-fix", conflicts_with = "min_width")]
    no_thin_fix: bool,
    /// The thin-feature fix's smallest width in mm, 0.25–1.2.
    #[arg(long = "min-width", default_value_t = 0.25)]
    min_width: f64,
    /// Which library fits the curves.
    #[arg(long, value_enum, default_value_t = FitterArg::Kurbo)]
    fitter: FitterArg,
    /// A saved edit log (JSON) to replay.
    #[arg(long)]
    edits: Option<PathBuf>,
    /// Don't print where the files were written. Warnings and errors still print.
    #[arg(short, long)]
    quiet: bool,
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("can't read {}: {e}", path.display()))
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| format!("can't write {}: {e}", path.display()))
}

/// Tells the user where each written file is, as a full path.
fn report_written(paths: &[&Path]) {
    println!("Files generated successfully:");
    for path in paths {
        // `absolute` doesn't touch the file system, so it can't fail on a file we just wrote
        // unless the working directory is gone; fall back to the path as given.
        let shown = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        println!("  {}", shown.display());
    }
}

fn run(args: Args) -> Result<(), String> {
    let image = read_png(&read(&args.input)?).map_err(|e| e.to_string())?;
    let edits = match &args.edits {
        Some(path) => serde_json::from_slice::<EditLog>(&read(path)?)
            .map_err(|e| format!("can't read the edit log {}: {e}", path.display()))?,
        None => EditLog::new(),
    };
    let job = Job {
        output_colours: args.output_colours,
        base: args.base,
        layer: args.layer,
        size_x: args.size_x,
        thin_fix: if args.no_thin_fix {
            ThinFix::Off
        } else {
            ThinFix::MinWidth(args.min_width)
        },
        edits,
        fitter: match args.fitter {
            FitterArg::Kurbo => Fitter::Kurbo,
            FitterArg::Visioncortex => Fitter::VisionCortex,
        },
        ..Job::new(image)
    };
    let slicer = match args.slicer {
        SlicerArg::Prusa => Slicer::Prusa,
        SlicerArg::Bambu => Slicer::Bambu,
    };
    let fail = |e: flatart_domain::Error| e.to_string();
    let mut pipeline = job.pipeline().map_err(fail)?;
    let name = args
        .input
        .file_stem()
        .map_or("FlatArt".into(), |s| s.to_string_lossy().into_owned());
    let options = ExportOptions {
        name,
        thumbnail: Some(pipeline.thumbnail(THUMBNAIL_SIDE).map_err(fail)?),
    };
    write(
        &args.output,
        &pipeline.export(slicer.exporter(), &options).map_err(fail)?,
    )?;
    let mut written = vec![args.output.as_path()];
    if let Some(svg) = &args.svg {
        write(svg, &pipeline.export("svg", &options).map_err(fail)?)?;
        written.push(svg);
    }
    if let Some(step) = &args.step {
        write(step, &pipeline.export("step", &options).map_err(fail)?)?;
        written.push(step);
    }
    let thin = pipeline.printable().map_err(fail)?.thin_features.len();
    if thin > 0 {
        eprintln!("warning: {thin} feature(s) are thinner than the minimum width");
    }
    for dropped in &pipeline.mapped().map_err(fail)?.dropped_edits {
        eprintln!("warning: an edit was dropped: {dropped:?}");
    }
    if !args.quiet {
        report_written(&written);
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("flatart: {message}");
            ExitCode::FAILURE
        }
    }
}
