//! Evaluate raw starting looks against each file's own embedded JPEG (docs/camera-preview-colour.md):
//!
//! - `camera_look_eval preview RAW OUT.jpg [EDGE]` writes the raw's embedded camera JPEG, oriented,
//!   as sRGB (≤ EDGE px, default 1200), for side-by-side viewing with a render of the raw;
//! - `camera_look_eval delta REFERENCE.jpg RENDER.jpg… [--edge N]` prints, for each render, the
//!   mean CIE76 ΔE and mean L* against the reference, both reduced to ≤ N px (default 200).
//!
//! Renders come from `lightcraft-cli render RAW -o OUT.jpg --size 1200`; `LIGHTCRAFT_PROFILE=1`
//! makes the render print the fit's alignment, held-out error and verdict.
use lightcraft_codecs::{DecodeOptions, decode};
use lightcraft_raster::resample::{Filter, fit};

fn lab(rgb2020: [f32; 3]) -> [f64; 3] {
    // linear Rec.2020 → XYZ (D65) → CIELAB (D65 white)
    let m = [[0.636958, 0.144617, 0.168881], [0.262700, 0.677998, 0.059302], [0.0, 0.028073, 1.060985]];
    let p = rgb2020.map(|v| f64::from(v).max(0.0));
    let xyz: [f64; 3] = std::array::from_fn(|i| m[i][0] * p[0] + m[i][1] * p[1] + m[i][2] * p[2]);
    let white = [0.95047, 1.0, 1.08883];
    let f = |t: f64| if t > 216.0 / 24389.0 { t.cbrt() } else { (24389.0 / 27.0 * t + 16.0) / 116.0 };
    let [fx, fy, fz] = std::array::from_fn(|i| f(xyz[i] / white[i]));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

fn load(path: &str, edge: usize) -> Result<lightcraft_raster::Rgb32f, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let d = decode(&bytes, DecodeOptions::fit(edge as u32 * 2, edge as u32 * 2)).map_err(|e| format!("{path}: {e}"))?;
    Ok(fit(&d.to_working(), edge, edge, Filter::Box))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("preview") if args.len() >= 3 => {
            let bytes = std::fs::read(&args[1]).map_err(|e| e.to_string())?;
            let edge = args.get(3).and_then(|e| e.parse().ok()).unwrap_or(1200);
            let img = lightcraft_engine::files::embedded_preview_srgb(&bytes, edge).ok_or("no embedded preview")?;
            let jpeg = lightcraft_preview::encode_jpeg(&img).ok_or("can't encode")?;
            std::fs::write(&args[2], jpeg).map_err(|e| e.to_string())
        }
        Some("delta") if args.len() >= 3 => {
            let mut edge = 200;
            let mut files = Vec::new();
            let mut it = args[1..].iter();
            while let Some(a) = it.next() {
                if a == "--edge" {
                    edge = it.next().and_then(|v| v.parse().ok()).ok_or("--edge N")?;
                } else {
                    files.push(a.clone());
                }
            }
            let reference = load(&files[0], edge)?;
            for path in &files[1..] {
                let render = load(path, edge)?;
                let render = lightcraft_raster::resample::resize(&render, reference.width, reference.height, Filter::Box);
                let (mut de, mut l_ref, mut l_out) = (0.0, 0.0, 0.0);
                for (a, b) in reference.data.iter().zip(&render.data) {
                    let (a, b) = (lab(*a), lab(*b));
                    de += ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                    l_ref += a[0];
                    l_out += b[0];
                }
                let n = reference.data.len().max(1) as f64;
                println!("{path}\tdE76 {:.2}\tL* {:.1} (reference {:.1})", de / n, l_out / n, l_ref / n);
            }
            Ok(())
        }
        _ => Err("usage: camera_look_eval preview RAW OUT.jpg [EDGE] | delta REFERENCE.jpg RENDER.jpg… [--edge N]".into()),
    }
}
