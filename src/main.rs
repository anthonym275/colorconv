mod color;

use color::{Lab, Rgb};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let result = match args.first().map(String::as_str) {
        Some("rgb-to-lab") => run_rgb_to_lab(&args[1..]),
        Some("lab-to-rgb") => run_lab_to_rgb(&args[1..]),
        Some("-h") | Some("--help") | None => {
            print_usage();
            return ExitCode::SUCCESS;
        }
        Some(other) => Err(format!("unknown command {:?}", other)),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run_rgb_to_lab(args: &[String]) -> Result<(), String> {
    let hex = args.first().ok_or("usage: colorconv rgb-to-lab <hex>")?;
    let rgb = Rgb::from_hex(hex)?;
    let lab = rgb.to_lab();
    println!("L: {:.2}  a: {:.2}  b: {:.2}", lab.l, lab.a, lab.b);
    Ok(())
}

fn run_lab_to_rgb(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err("usage: colorconv lab-to-rgb <L> <a> <b>".to_string());
    }
    let parse = |i: usize, name: &str| -> Result<f64, String> {
        args[i]
            .parse::<f64>()
            .map_err(|_| format!("{name} must be a number, got {:?}", args[i]))
    };
    let lab = Lab {
        l: parse(0, "L")?,
        a: parse(1, "a")?,
        b: parse(2, "b")?,
    };
    let rgb = lab.to_rgb();
    println!("#{}", rgb.to_hex());
    Ok(())
}

fn print_usage() {
    println!(
        "colorconv - convert between sRGB and CIE L*a*b*\n\n\
         usage:\n\
         \x20 colorconv rgb-to-lab <hex>\n\
         \x20 colorconv lab-to-rgb <L> <a> <b>\n\n\
         examples:\n\
         \x20 colorconv rgb-to-lab FF5733\n\
         \x20 colorconv lab-to-rgb 58.99 60.94 55.60"
    );
}
