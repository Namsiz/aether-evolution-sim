// Trace rem_pio2 + kernel_sin for a known-mismatch input.
fn main() {
    let x = 5.981306789687621f64;
    // high word
    let ix = ((x.to_bits() >> 32) as u32 & 0x7FFF_FFFF) as i32;
    println!("ix=0x{ix:08x}");
    // replicating rem_pio2 medium branch manually
    const INVPIO2: f64 = f64::from_bits(0x3FE4_5F30_6DC9_C883);
    const PIO2_1: f64 = f64::from_bits(0x3FF9_21FB_5440_0000);
    const PIO2_1T: f64 = f64::from_bits(0x3DD0_B461_1A62_6331);
    let t = x.abs();
    let n = (t * INVPIO2 + 0.5) as i32;
    let f_n = n as f64;
    let r = t - f_n * PIO2_1;
    let w = f_n * PIO2_1T;
    println!("n={n} r={r:e} w={w:e}");
    let y0 = r - w;
    let y1 = (r - y0) - w;
    println!("y0={y0:e} 0x{:016x}", y0.to_bits());
    println!("y1={y1:e} 0x{:016x}", y1.to_bits());
    // kernel_sin(y0, y1, 1)
    const HALF: f64 = 0.5;
    const S1: f64 = f64::from_bits(0xBFC5_5555_5555_5549);
    const S2: f64 = f64::from_bits(0x3F81_1111_1110_F8A6);
    const S3: f64 = f64::from_bits(0xBF2A_01A0_19C1_61D5);
    const S4: f64 = f64::from_bits(0x3EC7_1DE3_57B1_FE7D);
    const S5: f64 = f64::from_bits(0xBE5A_E5E6_8A2B_9CEB);
    const S6: f64 = f64::from_bits(0x3DE5_D93A_5ACF_D57C);
    let z = y0 * y0;
    let v = z * y0;
    let rr = S2 + z * (S3 + z * (S4 + z * (S5 + z * S6)));
    let res = y0 - ((z * (HALF * y1 - v * rr) - y1) - v * S1);
    println!("z={z:e} v={v:e} r={rr:e}");
    println!("kernel_sin={res:e} 0x{:016x}", res.to_bits());
    println!("aether_libm::sin = 0x{:016x}", aether_libm::sin(x).to_bits());
    println!("want            = 0xbfd307328f4fa3ca");
}
