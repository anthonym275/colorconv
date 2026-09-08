mod color;

use color::{Hsl, Lab, Rgb};
use std::env;
use std::io::{self, BufRead};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let result = match args.first().map(String::as_str) {
        Some("rgb-to-lab") => run_rgb_to_lab(&args[1..]),
        Some("lab-to-rgb") => run_lab_to_rgb(&args[1..]),
        Some("rgb-to-hsl") => run_rgb_to_hsl(&args[1..]),
        Some("hsl-to-rgb") => run_hsl_to_rgb(&args[1..]),
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
    match args.first() {
        Some(hex) => {
            let rgb = Rgb::from_hex(hex)?;
            let lab = rgb.to_lab();
            println!("L: {:.2}  a: {:.2}  b: {:.2}", lab.l, lab.a, lab.b);
            Ok(())
        }
        // No hex on the command line: treat stdin as a batch, one hex code
        // per line, so a whole palette can be piped through at once.
        None => run_rgb_to_lab_batch(),
    }
}

fn run_rgb_to_lab_batch() -> Result<(), String> {
    let stdin = io::stdin();
    let mut had_error = false;
    for (i, line) in stdin.lock().lines().enumerate() {
        let line = line.map_err(|e| format!("failed to read stdin: {e}"))?;
        let hex = line.trim();
        if hex.is_empty() {
            continue;
        }
        match Rgb::from_hex(hex) {
            Ok(rgb) => {
                let lab = rgb.to_lab();
                println!("{hex}  L: {:.2}  a: {:.2}  b: {:.2}", lab.l, lab.a, lab.b);
            }
            Err(msg) => {
                eprintln!("line {}: {msg}", i + 1);
                had_error = true;
            }
        }
    }
    if had_error {
        Err("one or more lines failed to parse".to_string())
    } else {
        Ok(())
    }
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

fn run_rgb_to_hsl(args: &[String]) -> Result<(), String> {
    let hex = args.first().ok_or("usage: colorconv rgb-to-hsl <hex>")?;
    let rgb = Rgb::from_hex(hex)?;
    let hsl = rgb.to_hsl();
    println!("H: {:.2}  S: {:.2}%  L: {:.2}%", hsl.h, hsl.s, hsl.l);
    Ok(())
}

fn run_hsl_to_rgb(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err("usage: colorconv hsl-to-rgb <H> <S> <L>".to_string());
    }
    let parse = |i: usize, name: &str| -> Result<f64, String> {
        args[i]
            .parse::<f64>()
            .map_err(|_| format!("{name} must be a number, got {:?}", args[i]))
    };
    let hsl = Hsl {
        h: parse(0, "H")?,
        s: parse(1, "S")?,
        l: parse(2, "L")?,
    };
    let rgb = hsl.to_rgb();
    println!("#{}", rgb.to_hex());
    Ok(())
}

fn print_usage() {
    println!(
        "colorconv - convert between sRGB, HSL, and CIE L*a*b*\n\n\
         usage:\n\
         \x20 colorconv rgb-to-lab <hex>\n\
         \x20 colorconv rgb-to-lab            (reads hex codes from stdin, one per line)\n\
         \x20 colorconv lab-to-rgb <L> <a> <b>\n\
         \x20 colorconv rgb-to-hsl <hex>\n\
         \x20 colorconv hsl-to-rgb <H> <S> <L>\n\n\
         examples:\n\
         \x20 colorconv rgb-to-lab FF5733\n\
         \x20 colorconv lab-to-rgb 58.99 60.94 55.60\n\
         \x20 colorconv rgb-to-hsl FF5733\n\
         \x20 colorconv hsl-to-rgb 10.59 100 60\n\
         \x20 printf 'FF5733\\n000000\\n' | colorconv rgb-to-lab"
    );
}
