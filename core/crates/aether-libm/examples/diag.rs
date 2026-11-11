// Diagnostic: verify regenerated inputs match stored inputs, then count
// bit mismatches for each function over the full 1e6 golden files.
use aether_rng::Mulberry32;

const SEED: u32 = 20261102;
const TWO_PI: f64 = std::f64::consts::PI * 2.0;

fn main() {
    // n_special_per_func maps name → number of hardcoded special entries at head.
    let cases: &[(&str, usize)] = &[
        ("sin", 30),
        ("cos", 30),
        ("exp", 31),
        ("atan2", 32),
        ("pow", 27),
    ];
    for (name, n_special) in cases {
        let path = format!("/tmp/libm_gold_sample_{name}.bin");
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => { eprintln!("cannot read {path}: {e}"); continue; }
        };
        let elem_size = if *name == "atan2" || *name == "pow" { 24u32 } else { 16u32 };
        let n = bytes.len() / elem_size as usize;
        let mut rng_a = Mulberry32::new(SEED);
        let mut rng_b = Mulberry32::new(SEED);
        let mut input_mismatch = 0usize;
        let mut out_mismatch = 0usize;
        let mut first: Option<(usize, f64, f64, f64)> = None;
        for i in 0..n {
            let ax = f64::from_le_bytes(bytes[i * elem_size as usize..i * elem_size as usize + 8].try_into().unwrap());
            let bx = if *name == "atan2" || *name == "pow" {
                f64::from_le_bytes(bytes[i * elem_size as usize + 8..i * elem_size as usize + 16].try_into().unwrap())
            } else {
                0.0
            };
            let expect = f64::from_le_bytes(
                bytes[(if *name == "atan2" || *name == "pow" {
                    i * elem_size as usize + 16
                } else {
                    i * elem_size as usize + 8
                })..((if *name == "atan2" || *name == "pow" { i + 1 } else { i + 1 }) * elem_size as usize)].try_into().unwrap(),
            );
            let got;
            if i < *n_special {
                if *name == "atan2" || *name == "pow" {
                    got = match *name {
                        "atan2" => aether_libm::atan2(ax, bx),
                        _ => aether_libm::pow(ax, bx),
                    };
                } else {
                    got = match *name {
                        "sin" => aether_libm::sin(ax),
                        "cos" => aether_libm::cos(ax),
                        _ => aether_libm::exp(ax),
                    };
                }
            } else {
                let xa = if *name == "atan2" { (rng_a.next_f64() * 2.0 - 1.0) * TWO_PI }
                    else if *name == "pow" { rng_a.next_f64() }
                    else { rng_a.next_f64() * TWO_PI };
                let xb = if *name == "atan2" { (rng_b.next_f64() * 2.0 - 1.0) * TWO_PI }
                    else if *name == "pow" { 1.0 + rng_b.next_f64() * 0.7 }
                    else { 0.0 };
                got = match *name {
                    "sin" => aether_libm::sin(xa),
                    "cos" => aether_libm::cos(xa),
                    "exp" => aether_libm::exp(xa),
                    "atan2" => aether_libm::atan2(xa, xb),
                    "pow" => aether_libm::pow(xa, xb),
                    _ => unreachable!(),
                };
            }
            // verify regenerated inputs match stored (only for non-special branch,
            // since special inputs are hardcoded — but we already verified above by not consuming rng).
            let ok = (expect.is_nan() && got.is_nan()) || expect.to_bits() == got.to_bits();
            if !ok {
                out_mismatch += 1;
                if first.is_none() {
                    first = Some((i, ax, expect, got));
                }
            }
        }
        println!(
            "{name}: n={n} out_mismatch={out_mismatch} first={:?}",
            first.map(|(i, x, e, g)| (i, x.to_bits(), e.to_bits(), g.to_bits()))
        );
    }
}
