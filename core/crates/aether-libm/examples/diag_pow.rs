fn set_high_word(x: f64, hi: u32) -> f64 {
    f64::from_bits(((hi as u64) << 32) | (x.to_bits() & 0xFFFF_FFFF))
}
fn main() {
    let x = 0.7f64; let y = 1.35f64;
    let hx = (x.to_bits() >> 32) as i32;
    let lx = x.to_bits() as u32;
    let hy = (y.to_bits() >> 32) as i32;
    let ly = y.to_bits() as u32;
    let mut ix = (hx as u32) & 0x7fff_ffff;
    let iy = (hy as u32) & 0x7fff_ffff;
    let mut n: i32 = (hx >> 31) + 1;
    let mut ax = x.abs();
    n = 0;
    n += ((ix >> 20) as i32) - 0x3ff;
    let j = (ix & 0x000f_ffff) as usize;
    ix = j as u32 | 0x3ff0_0000;
    let k: usize = if j <= 0x3988E { 0 } else if j < 0xBB67A { 1 } else { 0 };
    if j >= 0xBB67A { n += 1; ix -= 0x0010_0000; }
    println!("n={} j=0x{:x} k={} ix=0x{:08x}", n, j, k, ix);
    ax = set_high_word(ax, ix);
    println!("ax={:e}", ax);
    const BP: [f64; 2] = [1.0, 1.5];
    let u = ax - BP[k];
    let v = 1.0 / (ax + BP[k]);
    let ss = u * v;
    let mut s_h = ss;
    s_h = f64::from_bits(s_h.to_bits() & 0xFFFF_FFFF_0000_0000);
    let high = ((((ix as u32) >> 1) | 0x2000_0000) + 0x0008_0000 + ((k as u32) << 18));
    let mut t_h = f64::from_bits((high as u64) << 32);
    let t_l = ax - (t_h - BP[k]);
    let s_l = v * ((u - s_h * t_h) - s_h * t_l);
    println!("u={u:e} v={v:e} ss={ss:e} s_h={s_h:e} s_l={s_l:e} t_h={t_h:e} t_l={t_l:e}");
    const L1: f64 = 5.999_999_999_999_946_487_25e-01;
    const L2: f64 = 4.285_714_285_785_501_842_52e-01;
    const L3: f64 = 3.333_333_298_183_774_329_18e-01;
    const L4: f64 = 2.727_281_238_085_340_064_89e-01;
    const L5: f64 = 2.306_607_457_755_617_540_67e-01;
    const L6: f64 = 2.069_750_178_003_384_177_84e-01;
    let mut s2 = ss * ss;
    let mut r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))));
    r += s_l * (s_h + ss);
    s2 = s_h * s_h;
    t_h = 3.0 + s2 + r;
    t_h = f64::from_bits(t_h.to_bits() & 0xFFFF_FFFF_0000_0000);
    let t_l = r - ((t_h - 3.0) - s2);
    println!("r={r:e} s2={s2:e} t_h={t_h:e} t_l={t_l:e}");
    let u = s_h * t_h;
    let v = s_l * t_h + t_l * ss;
    let mut p_h = u + v;
    p_h = f64::from_bits(p_h.to_bits() & 0xFFFF_FFFF_0000_0000);
    let p_l = v - (p_h - u);
    println!("u={u:e} v={v:e} p_h={p_h:e} p_l={p_l:e}");
    // z_h, z_l, t1, t2 (log section outputs)
    const CP_H: f64 = 9.617_967_009_544_372_558_59e-01;
    const CP_L: f64 = -7.028_461_650_952_758_265_16e-09;
    const CP: f64 = 9.617_966_939_259_755_543_29e-01;
    const DP_H: [f64; 2] = [0.0, 5.849_624_872_207_641_601_56e-01];
    const DP_L: [f64; 2] = [0.0, 1.350_039_202_129_748_971_28e-08];
    let z_h = CP_H * p_h;
    let z_l = CP_L * p_h + p_l * CP + DP_L[k];
    let tt = n as f64;
    let t1 = ((z_h + z_l) + DP_H[k]) + tt;
    let t1 = f64::from_bits(t1.to_bits() & 0xFFFF_FFFF_0000_0000);
    let t2 = z_l - (((t1 - tt) - DP_H[k]) - z_h);
    println!("z_h={z_h:e} z_l={z_l:e} t1={t1:e} t2={t2:e}");
    let _ = ly;
    let _ = iy;
    let _ = lx;
    let _ = hy;
    part2(y, t1, t2);
}

// ---- continued: y-split + 2**(p_h+p_l) ----
#[allow(dead_code)]
fn part2(y: f64, t1: f64, t2: f64) -> f64 {
    let y1 = f64::from_bits(y.to_bits() & 0xFFFF_FFFF_0000_0000);
    println!("y1={y1:e}");
    let p_l = (y - y1) * t1 + y * t2;
    let p_h = y1 * t1;
    let z = p_l + p_h;
    println!("p_l={p_l:e} p_h={p_h:e} z={z:e}");
    let j = (z.to_bits() >> 32) as i32;
    let i = (z.to_bits() & 0xFFFF_FFFF) as i32;
    println!("j={j} (0x{j:08x}) i={i}");
    let i = j & 0x7fff_ffff;
    let mut k = (i >> 20) - 0x3ff;
    let mut n = 0i32;
    let mut p_h = p_h;
    if i > 0x3fe0_0000 {
        n = j + (0x0010_0000 >> (k + 1));
        k = ((n & 0x7fff_ffff) >> 20) - 0x3ff;
        let t = f64::from_bits(((n & !(0x000f_ffff >> k)) as u64) << 32);
        n = ((n & 0x000f_ffff) | 0x0010_0000) >> (20 - k);
        if j < 0 { n = -n; }
        println!("adjust: n={n} k={k} t={t:e}");
        p_h -= t;
    }
    println!("after adjust p_h={p_h:e} p_l={p_l:e}");
    let mut t = p_l + p_h;
    t = f64::from_bits(t.to_bits() & 0xFFFF_FFFF_0000_0000);
    println!("t(p_l+p_h, low=0)={t:e}");
    const LG2_H: f64 = 6.931_471_824_645_996_093_75e-01;
    const LG2: f64 = 6.931_471_805_599_452_862_27e-01;
    const LG2_L: f64 = -1.904_654_299_957_768_045_25e-09;
    const P1: f64 = 1.666_666_666_666_660_190_37e-01;
    const P2: f64 = -2.777_777_777_701_559_338_42e-03;
    const P3: f64 = 6.613_756_321_437_934_361_17e-05;
    const P4: f64 = -1.653_390_220_546_525_153_90e-06;
    const P5: f64 = 4.138_136_797_057_238_460_39e-08;
    let u = t * LG2_H;
    let v = (p_l - (t - p_h)) * LG2 + t * LG2_L;
    let z = u + v;
    let w = v - (z - u);
    println!("u={u:e} v={v:e} z={z:e} w={w:e}");
    let t = z * z;
    let t1 = z - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    let r = (z * t1) / ((t1 - 2.0) - (w + z * w));
    let z = 1.0 - (r - z);
    println!("t={t:e} t1={t1:e} r={r:e} final z={z:e}");
    z
}
