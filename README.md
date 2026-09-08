# colorconv

A small command-line tool that converts colors between sRGB (hex), HSL,
and CIE L\*a\*b\*.

Hex codes are what you paste into CSS or a design tool, but they're a poor
space for anything that involves *comparing* colors - RGB distance doesn't
match how different two colors actually look to a human eye. Lab was built
so that Euclidean distance between two points corresponds reasonably well to
perceived difference. If you're deduplicating a palette, sorting swatches by
similarity, or checking whether a color is "close enough" to a brand color,
you want to do that math in Lab, not RGB.

This tool does the round trip: sRGB -> linear RGB -> CIE XYZ -> Lab, and
back. No dependencies, no config, just the conversion.

HSL is included too, mostly because it's the format designers actually
type into a color picker. It's not perceptually uniform the way Lab is -
don't use HSL distance for the same comparisons Lab is good at - but
having it saves a trip to a separate tool when you just need to eyeball
or tweak a color.

## Usage

```
colorconv rgb-to-lab FF5733
L: 58.99  a: 60.94  b: 55.60

colorconv lab-to-rgb 58.99 60.94 55.60
#FF5733
```

The hex argument accepts an optional leading `#`. Lab values are plain
floats; L is expected in 0..100, a and b are unbounded but meaningful
values sit roughly in -128..127.

Running `rgb-to-lab` with no hex argument reads hex codes from stdin
instead, one per line, and prints one result line per input:

```
printf 'FF5733\n000000\n' | colorconv rgb-to-lab
FF5733  L: 58.99  a: 60.94  b: 55.60
000000  L: 0.00  a: 0.00  b: 0.00
```

Blank lines are skipped. A line that doesn't parse is reported on stderr
with its line number and the rest of the batch still runs; the process
exits non-zero if any line failed.

Round-tripping isn't always exact: Lab covers colors outside the sRGB
gamut, so converting an out-of-gamut Lab value back to RGB clamps to the
nearest representable color rather than failing.

HSL works the same way, just without the batch mode:

```
colorconv rgb-to-hsl FF5733
H: 10.59  S: 100.00%  L: 60.00%

colorconv hsl-to-rgb 10.59 100 60
#FF5733
```

## Building

Requires only the Rust standard library.

```
cargo build --release
./target/release/colorconv rgb-to-lab 336699
```

## How it works

`src/color.rs` has the actual math: the sRGB gamma curve, the sRGB/XYZ
matrices (D65 white point, 2-degree observer), the XYZ/Lab piecewise
functions from the CIE spec, and the HSL conversions (the standard
min/max-channel formulas, same as what's in the CSS Color spec).
`src/main.rs` wraps those conversions with argument parsing and the
stdin batch mode.

## License

MIT, see LICENSE.
