use aether_rng::Mulberry32;
const SEED: u32 = 20261102;
fn main() {
    let bytes = std::fs::read("/tmp/libm_gold_sample_sin.bin").unwrap();
    let mut rng = Mulberry32::new(SEED);
    let n_special = 30;
    let n = bytes.len() / 16;
    let mut shown = 0;
    for i in 0..n {
        let off = i * 16;
        let sx = f64::from_le_bytes(bytes[off..off+8].try_into().unwrap());
        let expect = f64::from_le_bytes(bytes[off+8..off+16].try_into().unwrap());
        let x = if i < n_special { sx } else { rng.next_f64() * std::f64::consts::PI * 2.0 };
        let got = aether_libm::sin(x);
        if x.to_bits() != sx.to_bits() && i >= n_special {
            println!("INPUT MISALIGN at i={i}: stored=0x{:016x} regen=0x{:016x}", sx.to_bits(), x.to_bits());
            break;
        }
        if got.to_bits() != expect.to_bits() && !(got.is_nan() && expect.is_nan()) {
            println!("i={i} x={x:e} 0x{:016x} exp=0x{:016x} ({}e) got=0x{:016x} ({}e)", x.to_bits(), expect.to_bits(), expect, got.to_bits(), got);
            shown += 1;
            if shown >= 5 { break; }
        }
    }
    println!("done shown={shown}");
}
