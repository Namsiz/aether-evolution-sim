use aether_rng::Mulberry32;
fn main() {
    let bytes = std::fs::read("/tmp/libm_gold_sample_pow.bin").unwrap();
    let n_special = 27;
    let mut rng_a = Mulberry32::new(20261102);
    let mut rng_b = Mulberry32::new(20261102);
    for i in 0..70 {
        let (a, b) = if i < n_special {
            let a = f64::from_le_bytes(bytes[i*24..i*24+8].try_into().unwrap());
            let b = f64::from_le_bytes(bytes[i*24+8..i*24+16].try_into().unwrap());
            (a, b)
        } else {
            (rng_a.next_f64(), 1.0 + rng_b.next_f64() * 0.7)
        };
        let sa = f64::from_le_bytes(bytes[i*24..i*24+8].try_into().unwrap());
        let sb = f64::from_le_bytes(bytes[i*24+8..i*24+16].try_into().unwrap());
        let expect = f64::from_le_bytes(bytes[i*24+16..i*24+24].try_into().unwrap());
        let got = aether_libm::pow(a, b);
        let input_ok = a.to_bits() == sa.to_bits() && b.to_bits() == sb.to_bits();
        if got.to_bits() != expect.to_bits() || !input_ok {
            println!("i={i} input_ok={input_ok}");
            println!("  a={a:e} (stored {sa:e}) a_ok={}", a.to_bits()==sa.to_bits());
            println!("  b={b:e} (stored {sb:e}) b_ok={}", b.to_bits()==sb.to_bits());
            println!("  got=0x{:016x} expect=0x{:016x}", got.to_bits(), expect.to_bits());
            if i > 63 { break; }
        }
    }
}
