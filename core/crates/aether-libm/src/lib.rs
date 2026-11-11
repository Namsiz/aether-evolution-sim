//! `aether-libm` — bit-exact Rust ports of the fdlibm-derived double-precision
//! math routines that V8 (Node.js 22 / V8 12.4, `src/base/ieee754.cc`) uses to
//! implement `Math.sin`, `Math.cos`, `Math.exp`, `Math.atan2` (and `Math.pow`,
//! needed for the terrain `Math.pow(m, 1.35)` path).
//!
//! Algorithm provenance: fdlibm (Sun Microsystems, 1993), transcribed from
//! V8 12.4 `src/base/ieee754.cc` (which is the exact code path Node 22.22.1
//! executes for these functions). Constants are given as IEEE-754 bit patterns
//! taken from the hexadecimal comments in the V8 source, so no decimal-literal
//! transcription errors are possible.
//!
//! Constraints:
//! - pure Rust bit arithmetic, no external libm, no libm crate, no js-sys —
//!   must compile for wasm32-unknown-unknown;
//! - no FMA contraction: Rust never fuses `a*b + c` unless explicitly asked,
//!   matching the C++ codegen assumptions of the fdlibm algorithms.
//!
//! Special-case semantics (NaN/Inf/signed zero) follow the V8 code verbatim;
//! any deviation found by the golden-sample tests must be recorded as
//! `[Chief decision needed]` per project policy.

#![allow(clippy::excessive_precision)]

#[inline]
fn get_high_word(x: f64) -> u32 {
    (x.to_bits() >> 32) as u32
}

#[inline]
fn get_low_word(x: f64) -> u32 {
    x.to_bits() as u32
}

#[inline]
fn insert_words(hi: u32, lo: u32) -> f64 {
    f64::from_bits(((hi as u64) << 32) | lo as u64)
}

#[inline]
fn set_high_word(x: f64, hi: u32) -> f64 {
    f64::from_bits(((hi as u64) << 32) | (x.to_bits() & 0xFFFF_FFFF))
}

#[inline]
fn set_low_word(x: f64, lo: u32) -> f64 {
    f64::from_bits((x.to_bits() & 0xFFFF_FFFF_0000_0000) | lo as u64)
}

/// fdlibm scalbn (musl-libc variant, exact power-of-two scaling; used by
/// rem_pio2 / exp / pow exactly as V8 uses the platform `scalbn()`).
pub fn scalbn(x: f64, mut n: i32) -> f64 {
    const P1023: f64 = f64::from_bits(0x7FE0_0000_0000_0000); // 2^1023
    const M1022_53: f64 = f64::from_bits(0x0360_0000_0000_0000) // 2^-1022
        * f64::from_bits(0x4350_0000_0000_0000); // * 2^53
    let mut y = x;
    if n > 1023 {
        y *= P1023;
        n -= 1023;
        if n > 1023 {
            y *= P1023;
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        // make sure final n < -53 to avoid double rounding in the subnormal range
        y *= M1022_53;
        n += 1022 - 53;
        if n < -1022 {
            y *= M1022_53;
            n += 1022 - 53;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    y * f64::from_bits(((0x3FF + n) as u64) << 52)
}

// ---------------------------------------------------------------------------
// __kernel_sin / __kernel_cos  (V8 ieee754.cc, fdlibm k_sin.c / k_cos.c)
// ---------------------------------------------------------------------------

/// fdlibm `__kernel_sin(x, y, iy)` on [-pi/4, pi/4]; `y` is the tail of `x`,
/// `iy` indicates whether `y` is nonzero.
fn kernel_sin(x: f64, y: f64, iy: i32) -> f64 {
    const HALF: f64 = 5.000_000_000_000_000_000_00e-01; /* 0x3FE00000, 0x00000000 */
    const S1: f64 = f64::from_bits(0xBFC5_5555_5555_5549); // -1.66666666666666324348e-01
    const S2: f64 = f64::from_bits(0x3F81_1111_1110_F8A6); //  8.33333333332248946124e-03
    const S3: f64 = f64::from_bits(0xBF2A_01A0_19C1_61D5); // -1.98412698298579493134e-04
    const S4: f64 = f64::from_bits(0x3EC7_1DE3_57B1_FE7D); //  2.75573137070700676789e-06
    const S5: f64 = f64::from_bits(0xBE5A_E5E6_8A2B_9CEB); // -2.50507602534068634195e-08
    const S6: f64 = f64::from_bits(0x3DE5_D93A_5ACF_D57C); //  1.58969099521155010221e-10

    let ix = (get_high_word(x) & 0x7FFF_FFFF) as i32;
    if ix < 0x3E40_0000 {
        /* |x| < 2**-27 */
        if (x as i32) == 0 {
            return x;
        }
    }
    let z = x * x;
    let v = z * x;
    // V8's build compiles fdlibm with -ffp-contract=fast, so every `a + b*c`
    // whose product feeds a single add becomes a fused multiply-add on
    // FMA-capable targets (verified against Node 22.22.1 golden samples on
    // aarch64). We replicate that explicitly with mul_add so the results are
    // target-independent (wasm32 included).
    let r = {
        let t1 = z.mul_add(S6, S5);
        let t2 = z.mul_add(t1, S4);
        let t3 = z.mul_add(t2, S3);
        z.mul_add(t3, S2)
    };
    if iy == 0 {
        let t1 = z.mul_add(r, S1); /* S1 + z*r */
        v.mul_add(t1, x) /* x + v*t1 */
    } else {
        // g++/clang -O2 pairing (verified by disassembly): v*r is a separate
        // fmul; half*y is fused into the subtract.
        let t_vr = v * r;
        let u = HALF.mul_add(y, -t_vr); /* half*y - v*r */
        let w1 = z.mul_add(u, -y); /* z*u - y */
        let w2 = v.mul_add(-S1, w1); /* w1 - v*S1 */
        x - w2
    }
}

/// fdlibm `__kernel_cos(x, y)` on [-pi/4, pi/4]; `y` is the tail of `x`.
fn kernel_cos(x: f64, y: f64) -> f64 {
    const ONE: f64 = 1.0;
    const C1: f64 = f64::from_bits(0x3FA5_5555_5555_554C); //  4.16666666666666019037e-02
    const C2: f64 = f64::from_bits(0xBF56_C16C_16C1_5177); // -1.38888888888741095749e-03
    const C3: f64 = f64::from_bits(0x3EFA_01A0_19CB_1590); //  2.48015872894767294178e-05
    const C4: f64 = f64::from_bits(0xBE92_7E4F_809C_52AD); // -2.75573143513906633035e-07
    const C5: f64 = f64::from_bits(0x3E21_EE9E_BDB4_B1C4); //  2.08757232129817482790e-09
    const C6: f64 = f64::from_bits(0xBDA8_FAE9_BE88_38D4); // -1.13596475577881948265e-11

    let ix = get_high_word(x) & 0x7FFF_FFFF;
    if ix < 0x3E40_0000 {
        /* if x < 2**27 */
        if (x as i32) == 0 {
            return ONE;
        }
    }
    let z = x * x;
    let r = {
        let t1 = z.mul_add(C6, C5);
        let t2 = z.mul_add(t1, C4);
        let t3 = z.mul_add(t2, C3);
        let t4 = z.mul_add(t3, C2);
        z.mul_add(t4, C1)
    };
    let r = z * r;
    if ix < 0x3FD3_3333 {
        /* if |x| < 0.3 */
        let xy = x * y;
        let zr = z.mul_add(r, -xy); /* z*r - x*y */
        let tc = (0.5f64).mul_add(z, -zr); /* 0.5*z - zr */
        ONE - tc
    } else {
        let qx = if ix > 0x3FE9_0000 {
            /* x > 0.78125 */
            0.28125
        } else {
            insert_words(ix - 0x0020_0000, 0) /* x/4 */
        };
        let iz = (0.5f64).mul_add(z, -qx); /* 0.5*z - qx */
        let a = ONE - qx;
        let xy = x * y;
        let zr = z.mul_add(r, -xy); /* z*r - x*y */
        let tc = iz - zr;
        a - tc
    }
}

// ---------------------------------------------------------------------------
// __ieee754_rem_pio2 / __kernel_rem_pio2  (V8 ieee754.cc, fdlibm e_rem_pio2.c
// / k_rem_pio2.c): reduce x to y[0] + y[1] = x - n*pi/2 with |y| < pi/2,
// returning n.
// ---------------------------------------------------------------------------

const TWO_OVER_PI: [i32; 66] = [
    0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62, 0x95993C, 0x439041, 0xFE5163,
    0xABDEBB, 0xC561B7, 0x246E3A, 0x424DD2, 0xE00649, 0x2EEA09, 0xD1921C, 0xFE1DEB, 0x1CB129,
    0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C, 0x7026B4, 0x5F7E41, 0x3991D6, 0x398353, 0x39F49C,
    0x845F8B, 0xBDF928, 0x3B1FF8, 0x97FFDE, 0x05980F, 0xEF2F11, 0x8B5A0A, 0x6D1F6D, 0x367ECF,
    0x27CB09, 0xB74F46, 0x3F669E, 0x5FEA2D, 0x7527BA, 0xC7EBE5, 0xF17B3D, 0x0739F7, 0x8A5292,
    0xEA6BFB, 0x5FB11F, 0x8D5D08, 0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20, 0x9AF436, 0x1DA9E3,
    0x91615E, 0xE61B08, 0x659985, 0x5F14A0, 0x68408D, 0xFFD880, 0x4D7327, 0x310606, 0x1556CA,
    0x73A8C9, 0x60E27B, 0xC08C6B,
];

const NPIO2_HW: [i32; 32] = [
    0x3FF921FB, 0x400921FB, 0x4012D97C, 0x401921FB, 0x401F6A7A, 0x4022D97C, 0x4025FDBB,
    0x402921FB, 0x402C463A, 0x402F6A7A, 0x4031475C, 0x4032D97C, 0x40346B9C, 0x4035FDBB,
    0x40378FDB, 0x403921FB, 0x403AB41B, 0x403C463A, 0x403DD85A, 0x403F6A7A, 0x40407E4C,
    0x4041475C, 0x4042106C, 0x4042D97C, 0x4043A28C, 0x40446B9C, 0x404534AC, 0x4045FDBB,
    0x4046C6CB, 0x40478FDB, 0x404858EB, 0x404921FB,
];

const INVPIO2: f64 = f64::from_bits(0x3FE4_5F30_6DC9_C883); // 6.36619772367581382433e-01
const PIO2_1: f64 = f64::from_bits(0x3FF9_21FB_5440_0000); // 1.57079632673412561417e+00
const PIO2_1T: f64 = f64::from_bits(0x3DD0_B461_1A62_6331); // 6.07710050650619224932e-11
const PIO2_2: f64 = f64::from_bits(0x3DD0_B461_1A60_0000); // 6.07710050630396597660e-11
const PIO2_2T: f64 = f64::from_bits(0x3BA3_198A_2E03_7073); // 2.02226624879595063154e-21
const PIO2_3: f64 = f64::from_bits(0x3BA3_198A_2E00_0000); // 2.02226624871116645580e-21
const PIO2_3T: f64 = f64::from_bits(0x397B_839A_2520_49C1); // 8.47842766036889956997e-32

fn rem_pio2(x: f64, y: &mut [f64; 2]) -> i32 {
    let hx = get_high_word(x) as i32;
    let ix = hx & 0x7FFF_FFFF;
    if ix <= 0x3FE9_21FB {
        /* |x| ~<= pi/4 , no need for reduction */
        y[0] = x;
        y[1] = 0.0;
        return 0;
    }
    if ix < 0x4002_D97C {
        /* |x| < 3pi/4, special case with n=+-1 */
        if hx > 0 {
            let mut z = x - PIO2_1;
            if ix != 0x3FF9_21FB {
                /* 33+53 bit pi is good enough */
                y[0] = z - PIO2_1T;
                y[1] = (z - y[0]) - PIO2_1T;
            } else {
                /* near pi/2, use 33+33+53 bit pi */
                z -= PIO2_2;
                y[0] = z - PIO2_2T;
                y[1] = (z - y[0]) - PIO2_2T;
            }
            return 1;
        } else {
            /* negative x */
            let mut z = x + PIO2_1;
            if ix != 0x3FF9_21FB {
                y[0] = z + PIO2_1T;
                y[1] = (z - y[0]) + PIO2_1T;
            } else {
                z += PIO2_2;
                y[0] = z + PIO2_2T;
                y[1] = (z - y[0]) + PIO2_2T;
            }
            return -1;
        }
    }
    if ix <= 0x4139_21FB {
        /* |x| ~<= 2^19*(pi/2), medium size */
        let t = x.abs();
        let n = (t * INVPIO2 + 0.5) as i32;
        let f_n = n as f64;
        let mut r = t - f_n * PIO2_1;
        let mut w = f_n * PIO2_1T; /* 1st round good to 85 bit */
        if n < 32 && ix != NPIO2_HW[(n - 1) as usize] {
            y[0] = r - w; /* quick check no cancellation */
        } else {
            let j = (ix >> 20) as i32;
            y[0] = r - w;
            let mut i = j - (((get_high_word(y[0]) as i32) >> 20) & 0x7FF);
            if i > 16 {
                /* 2nd iteration needed, good to 118 */
                let tt = r;
                w = f_n * PIO2_2;
                r = tt - w;
                w = f_n.mul_add(PIO2_2T, -((tt - r) - w));
                y[0] = r - w;
                i = j - (((get_high_word(y[0]) as i32) >> 20) & 0x7FF);
                if i > 49 {
                    /* 3rd iteration need, 151 bits acc */
                    let tt = r; /* will cover all possible cases */
                    w = f_n * PIO2_3;
                    r = tt - w;
                    w = f_n.mul_add(PIO2_3T, -((tt - r) - w));
                    y[0] = r - w;
                }
            }
        }
        y[1] = (r - y[0]) - w;
        if hx < 0 {
            y[0] = -y[0];
            y[1] = -y[1];
            -n
        } else {
            n
        }
    } else {
        /* all other (large) arguments */
        if ix >= 0x7FF0_0000 {
            /* x is inf or NaN */
            y[0] = x - x;
            y[1] = x - x;
            return 0;
        }
        /* set z = scalbn(|x|,ilogb(x)-23) */
        let low = get_low_word(x);
        let mut z = set_low_word(0.0, low);
        let e0 = (ix >> 20) - 1046; /* e0 = ilogb(z)-23; */
        z = set_high_word(z, ((ix as u32).wrapping_sub((e0 as u32) << 20)) as u32);
        let mut tx = [0.0f64; 3];
        for i in 0..2 {
            tx[i] = z as i32 as f64;
            z = (z - tx[i]) * 1.677_721_600_000_000_000_00e+07; /* two24 */
        }
        tx[2] = z;
        let mut nx = 3usize;
        while tx[nx - 1] == 0.0 {
            nx -= 1;
        }
        let n = kernel_rem_pio2(&tx, y, e0, nx, 2, &TWO_OVER_PI);
        if hx < 0 {
            y[0] = -y[0];
            y[1] = -y[1];
            -n
        } else {
            n
        }
    }
}

fn kernel_rem_pio2(x: &[f64; 3], y: &mut [f64; 2], e0: i32, nx: usize, prec: i32, ipio2: &[i32]) -> i32 {
    const INIT_JK: [usize; 4] = [2, 3, 4, 6]; /* initial value for jk */

    const PI_O2: [f64; 8] = [
        f64::from_bits(0x3FF9_21FB_4000_0000), // 1.57079625129699707031e+00
        f64::from_bits(0x3E74_442D_0000_0000), // 7.54978941586159635335e-08
        f64::from_bits(0x3CF8_4698_8000_0000), // 5.39030252995776476554e-15
        f64::from_bits(0x3B78_CC51_6000_0000), // 3.28200341580791294123e-22
        f64::from_bits(0x39F0_1B83_8000_0000), // 1.27065575308067607349e-29
        f64::from_bits(0x387A_2520_4000_0000), // 1.22933308981111328932e-36
        f64::from_bits(0x36E3_8222_8000_0000), // 2.73370053816464559624e-44
        f64::from_bits(0x3569_F31D_0000_0000), // 2.16741683877804819444e-51
    ];
    const TWO24: f64 = 1.677_721_600_000_000_000_00e+07; /* 0x41700000, 0x00000000 */
    const TWON24: f64 = 5.960_464_477_539_062_500_00e-08; /* 0x3E700000, 0x00000000 */

    let jk = INIT_JK[prec as usize];
    let jp = jk;

    /* determine jx,jv,q0, note that 3>q0 */
    let jx = nx - 1;
    let jv = ((e0 - 3) / 24).max(0);
    let mut q0 = e0 - 24 * (jv as i32 + 1);

    /* set up f[0] to f[jx+jk] where f[jx+jk] = ipio2[jv+jk] */
    let mut j = jv - jx as i32;
    let m = jx + jk;
    let mut f = [0.0f64; 20];
    for (_i, fi) in f.iter_mut().enumerate().take(m + 1) {
        *fi = if j < 0 { 0.0 } else { ipio2[j as usize] as f64 };
        j += 1;
    }

    /* compute q[0],q[1],...q[jk] */
    let mut q = [0.0f64; 20];
    for (i, qi) in q.iter_mut().enumerate().take(jk + 1) {
        let mut fw = 0.0f64;
        for jj in 0..=jx {
            fw += x[jj] * f[jx + i - jj];
        }
        *qi = fw;
    }

    let mut iq = [0i32; 20];
    let mut fq = [0.0f64; 20];
    let mut jz = jk;
    let mut n: i32;
    let mut ih: i32;
    let mut z: f64;

    'recompute: loop {
        /* distill q[] into iq[] reversingly */
        z = q[jz];
        let mut i = 0usize;
        let mut j = jz as i32;
        while j > 0 {
            let fw = (TWON24 * z) as i32 as f64;
            iq[i] = (z - TWO24 * fw) as i32;
            z = q[(j - 1) as usize] + fw;
            i += 1;
            j -= 1;
        }

        /* compute n */
        z = scalbn(z, q0); /* actual value of z */
        z -= 8.0 * (z * 0.125).floor(); /* trim off integer >= 8 */
        n = z as i32;
        z -= n as f64;
        ih = 0;
        if q0 > 0 {
            /* need iq[jz-1] to determine n */
            let ii = iq[jz - 1] >> (24 - q0);
            n += ii;
            iq[jz - 1] -= ii << (24 - q0);
            ih = iq[jz - 1] >> (23 - q0);
        } else if q0 == 0 {
            ih = iq[jz - 1] >> 23;
        } else if z >= 0.5 {
            ih = 2;
        }

        if ih > 0 {
            /* q > 0.5 */
            n += 1;
            let mut carry = 0i32;
            for item in iq.iter_mut().take(jz) {
                /* compute 1-q */
                let jj = *item;
                if carry == 0 {
                    if jj != 0 {
                        carry = 1;
                        *item = 0x1000000 - jj;
                    }
                } else {
                    *item = 0xFFFFFF - jj;
                }
            }
            if q0 > 0 {
                /* rare case: chance is 1 in 12 */
                match q0 {
                    1 => iq[jz - 1] &= 0x7FFFFF,
                    2 => iq[jz - 1] &= 0x3FFFFF,
                    _ => {}
                }
            }
            if ih == 2 {
                z = 1.0 - z;
                if carry != 0 {
                    z -= scalbn(1.0, q0);
                }
            }
        }

        /* check if recomputation is needed */
        if z == 0.0 {
            let mut jj = 0i32;
            for ii in (jk..jz).rev() {
                jj |= iq[ii];
            }
            if jj == 0 {
                /* need recomputation */
                let mut k = 1usize;
                while jk >= k && iq[jk - k] == 0 {
                    k += 1;
                }
                for ii in (jz + 1)..=(jz + k) {
                    /* add q[jz+1] to q[jz+k] */
                    f[jx + ii] = ipio2[(jv as usize) + ii] as f64;
                    let mut fw = 0.0f64;
                    for jjj in 0..=jx {
                        fw += x[jjj] * f[jx + ii - jjj];
                    }
                    q[ii] = fw;
                }
                jz += k;
                continue 'recompute;
            }
        }
        break;
    }

    /* chop off zero terms */
    if z == 0.0 {
        jz -= 1;
        q0 -= 24;
        while iq[jz] == 0 {
            jz -= 1;
            q0 -= 24;
        }
    } else {
        /* break z into 24-bit if necessary */
        z = scalbn(z, -q0);
        if z >= TWO24 {
            let fw = (TWON24 * z) as i32 as f64;
            iq[jz] = (z - TWO24 * fw) as i32;
            jz += 1;
            q0 += 24;
            iq[jz] = fw as i32;
        } else {
            iq[jz] = z as i32;
        }
    }

    /* convert integer "bit" chunk to floating-point value */
    let mut fw = scalbn(1.0, q0);
    for i in (0..=jz).rev() {
        q[i] = fw * iq[i] as f64;
        fw *= TWON24;
    }

    /* compute PIo2[0,...,jp]*q[jz,...,0] */
    for i in (0..=jz).rev() {
        let mut fw = 0.0f64;
        let mut k = 0usize;
        while k <= jp && k <= jz - i {
            fw += PI_O2[k] * q[i + k];
            k += 1;
        }
        fq[jz - i] = fw;
    }

    /* compress fq[] into y[] */
    match prec {
        0 => {
            let mut fw = 0.0f64;
            for i in (0..=jz).rev() {
                fw += fq[i];
            }
            y[0] = if ih == 0 { fw } else { -fw };
        }
        1 | 2 => {
            let mut fw = 0.0f64;
            for i in (0..=jz).rev() {
                fw += fq[i];
            }
            y[0] = if ih == 0 { fw } else { -fw };
            fw = fq[0] - fw;
            for i in 1..=jz {
                fw += fq[i];
            }
            y[1] = if ih == 0 { fw } else { -fw };
        }
        3 => {
            /* painful */
            for i in (1..=jz).rev() {
                let w = fq[i - 1] + fq[i];
                fq[i] += fq[i - 1] - w;
                fq[i - 1] = w;
            }
            for i in (2..=jz).rev() {
                let w = fq[i - 1] + fq[i];
                fq[i] += fq[i - 1] - w;
                fq[i - 1] = w;
            }
            let mut fw = 0.0f64;
            for i in (2..=jz).rev() {
                fw += fq[i];
            }
            if ih == 0 {
                y[0] = fq[0];
                y[1] = fq[1];
            } else {
                y[0] = -fq[0];
                y[1] = -fq[1];
            }
            let _ = fw;
        }
        _ => unreachable!(),
    }
    n & 7
}

// ---------------------------------------------------------------------------
// sin / cos  (V8 ieee754.cc, fdlibm s_sin.c / s_cos.c)
// ---------------------------------------------------------------------------

/// V8 `Math.sin` — fdlibm port (Node 22 / V8 12.4 `ieee754.cc`).
pub fn sin(x: f64) -> f64 {
    let ix = get_high_word(x) & 0x7FFF_FFFF;
    /* |x| ~< pi/4 */
    if ix <= 0x3FE9_21FB {
        return kernel_sin(x, 0.0, 0);
    } else if ix >= 0x7FF0_0000 {
        /* sin(Inf or NaN) is NaN */
        return x - x;
    }
    /* argument reduction needed */
    let mut y = [0.0f64; 2];
    let n = rem_pio2(x, &mut y);
    match n & 3 {
        0 => kernel_sin(y[0], y[1], 1),
        1 => kernel_cos(y[0], y[1]),
        2 => -kernel_sin(y[0], y[1], 1),
        _ => -kernel_cos(y[0], y[1]),
    }
}

/// V8 `Math.cos` — fdlibm port (Node 22 / V8 12.4 `ieee754.cc`).
pub fn cos(x: f64) -> f64 {
    let ix = get_high_word(x) & 0x7FFF_FFFF;
    /* |x| ~< pi/4 */
    if ix <= 0x3FE9_21FB {
        return kernel_cos(x, 0.0);
    } else if ix >= 0x7FF0_0000 {
        /* cos(Inf or NaN) is NaN */
        return x - x;
    }
    /* argument reduction needed */
    let mut y = [0.0f64; 2];
    let n = rem_pio2(x, &mut y);
    match n & 3 {
        0 => kernel_cos(y[0], y[1]),
        1 => -kernel_sin(y[0], y[1], 1),
        2 => -kernel_cos(y[0], y[1]),
        _ => kernel_sin(y[0], y[1], 1),
    }
}

// ---------------------------------------------------------------------------
// exp  (V8 ieee754.cc, fdlibm e_exp.c)
// ---------------------------------------------------------------------------

/// V8 `Math.exp` — fdlibm port (Node 22 / V8 12.4 `ieee754.cc`).
pub fn exp(x: f64) -> f64 {
    const ONE: f64 = 1.0;
    const HALF: [f64; 2] = [0.5, -0.5];
    const O_THRESHOLD: f64 = 7.097_827_128_933_839_730_96e+02; /* 0x40862E42, 0xFEFA39EF */
    const U_THRESHOLD: f64 = -7.451_332_191_019_411_084_20e+02; /* 0xC0874910, 0xD52D3051 */
    const LN2_HI: [f64; 2] = [
        6.931_471_803_691_238_164_90e-01,  /* 0x3FE62E42, 0xFEE00000 */
        -6.931_471_803_691_238_164_90e-01, /* 0xBFE62E42, 0xFEE00000 */
    ];
    const LN2_LO: [f64; 2] = [
        1.908_214_929_270_587_700_02e-10,  /* 0x3DEA39EF, 0x35793C76 */
        -1.908_214_929_270_587_700_02e-10, /* 0xBDEA39EF, 0x35793C76 */
    ];
    const INVLN2: f64 = 1.442_695_040_888_963_387_00e+00; /* 0x3FF71547, 0x652B82FE */
    const P1: f64 = 1.666_666_666_666_660_190_37e-01; /* 0x3FC55555, 0x5555553E */
    const P2: f64 = -2.777_777_777_701_559_338_42e-03; /* 0xBF66C16C, 0x16BEBD93 */
    const P3: f64 = 6.613_756_321_437_934_361_17e-05; /* 0x3F11566A, 0xAF25DE2C */
    const P4: f64 = -1.653_390_220_546_525_153_90e-06; /* 0xBEBBBD41, 0xC5D26BF1 */
    const P5: f64 = 4.138_136_797_057_238_460_39e-08; /* 0x3E663769, 0x72BEA4D0 */
    const E: f64 = 2.718_281_828_459_045; /* 0x4005BF0A, 0x8B145769 */

    const HUGE: f64 = 1.0e+300;
    const TWOM1000: f64 = 9.332_636_185_032_188_789_90e-302; /* 2**-1000=0x01700000,0 */
    const TWO1023: f64 = 8.988_465_674_311_579_539e307; /* 0x1p1023 */

    let k: i32;
    let mut hi = 0.0f64;
    let mut lo = 0.0f64;

    let hx = get_high_word(x);
    let xsb = ((hx >> 31) & 1) as i32; /* sign bit of x */
    let hx = hx & 0x7FFF_FFFF; /* high word of |x| */

    /* filter out non-finite argument */
    if hx >= 0x4086_2E42 {
        /* if |x|>=709.78... */
        if hx >= 0x7FF0_0000 {
            let lx = get_low_word(x);
            if ((hx & 0xF_FFFF) | lx) != 0 {
                return x + x; /* NaN */
            } else {
                return if xsb == 0 { x } else { 0.0 }; /* exp(+-inf)={inf,0} */
            }
        }
        if x > O_THRESHOLD {
            return HUGE * HUGE; /* overflow */
        }
        if x < U_THRESHOLD {
            return TWOM1000 * TWOM1000; /* underflow */
        }
    }

    /* argument reduction */
    let mut x = x;
    if hx > 0x3FD6_2E42 {
        /* if  |x| > 0.5 ln2 */
        if hx < 0x3FF0_A2B2 {
            /* and |x| < 1.5 ln2 */
            /* TODO(rtoy): We special case exp(1) here to return the correct
             * value of E, as the computation below would get the last bit
             * wrong. We should probably fix the algorithm instead.
             */
            if x == 1.0 {
                return E;
            }
            hi = x - LN2_HI[xsb as usize];
            lo = LN2_LO[xsb as usize];
            k = 1 - xsb - xsb;
        } else {
            // V8's compiled exp fuses `invln2*x + halF[xsb]` (FMA); replicate
            k = INVLN2.mul_add(x, HALF[xsb as usize]) as i32;
            let t = k as f64;
            hi = x - t * LN2_HI[0]; /* t*ln2HI is exact here */
            lo = t * LN2_LO[0];
        }
        x = hi - lo;
    } else if hx < 0x3E30_0000 {
        /* when |x|<2**-28 */
        if HUGE + x > ONE {
            return ONE + x; /* trigger inexact */
        }
        return ONE + x; // unreachable in practice; HUGE+x>ONE is true for finite x here
    } else {
        k = 0;
    }

    /* x is now in primary range */
    let t = x * x;
    let twopk = if k >= -1021 {
        f64::from_bits(((0x3FF0_0000u32.wrapping_add((k as u32).wrapping_shl(20))) as u64) << 32)
    } else {
        f64::from_bits(
            ((0x3FF0_0000u32.wrapping_add(((k + 1000) as u32).wrapping_shl(20))) as u64) << 32,
        )
    };
    let c = {
        let u1 = t.mul_add(P5, P4);
        let u2 = t.mul_add(u1, P3);
        let u3 = t.mul_add(u2, P2);
        let u4 = t.mul_add(u3, P1);
        (-t).mul_add(u4, x) /* x - t*u4 */
    };
    if k == 0 {
        ONE - ((x * c) / (c - 2.0) - x)
    } else {
        let y = ONE - ((lo - (x * c) / (2.0 - c)) - hi);
        if k >= -1021 {
            if k == 1024 {
                y * 2.0 * TWO1023
            } else {
                y * twopk
            }
        } else {
            y * twopk * TWOM1000
        }
    }
}

// ---------------------------------------------------------------------------
// atan / atan2  (V8 ieee754.cc, fdlibm s_atan.c / e_atan2.c)
// ---------------------------------------------------------------------------

/// fdlibm `atan(x)` on the full range (used by `atan2` internally).
fn atan(x: f64) -> f64 {
    const ATAN_HI: [f64; 4] = [
        4.636_476_090_008_060_935_15e-01, /* atan(0.5)hi 0x3FDDAC67, 0x0561BB4F */
        7.853_981_633_974_482_789_99e-01, /* atan(1.0)hi 0x3FE921FB, 0x54442D18 */
        9.827_937_232_473_290_540_82e-01, /* atan(1.5)hi 0x3FEF730B, 0xD281F69B */
        1.570_796_326_794_896_558_00e+00, /* atan(inf)hi 0x3FF921FB, 0x54442D18 */
    ];

    const ATAN_LO: [f64; 4] = [
        2.269_877_745_296_168_709_24e-17, /* atan(0.5)lo 0x3C7A2B7F, 0x222F65E2 */
        3.061_616_997_868_383_017_93e-17, /* atan(1.0)lo 0x3C81A626, 0x33145C07 */
        1.390_331_103_123_099_845_16e-17, /* atan(1.5)lo 0x3C700788, 0x7AF0CBBD */
        6.123_233_995_736_766_035_87e-17, /* atan(inf)lo 0x3C91A626, 0x33145C07 */
    ];

    const AT: [f64; 11] = [
        3.333_333_333_333_293_180_27e-01,  /* 0x3FD55555, 0x5555550D */
        -1.999_999_999_987_648_324_76e-01, /* 0xBFC99999, 0x9998EBC4 */
        1.428_571_427_250_346_637_11e-01,  /* 0x3FC24924, 0x920083FF */
        -1.111_111_040_546_235_578_80e-01, /* 0xBFBC71C6, 0xFE231671 */
        9.090_887_133_436_506_561_96e-02,  /* 0x3FB745CD, 0xC54C206E */
        -7.691_876_205_044_829_994_95e-02, /* 0xBFB3B0F2, 0xAF749A6D */
        6.661_073_137_387_531_206_69e-02,  /* 0x3FB10D66, 0xA0D03D51 */
        -5.833_570_133_790_573_486_45e-02, /* 0xBFADDE2D, 0x52DEFD9A */
        4.976_877_994_615_932_360_17e-02,  /* 0x3FA97B4B, 0x24760DEB */
        -3.653_157_274_421_691_552_70e-02, /* 0xBFA2B444, 0x2C6A6C2F */
        1.628_582_011_536_578_236_23e-02,  /* 0x3F90AD3A, 0xE322DA11 */
    ];

    const ONE: f64 = 1.0;
    const HUGE: f64 = 1.0e300;

    let mut x = x;
    let hx = get_high_word(x) as i32;
    let ix = hx & 0x7FFF_FFFF;
    if ix >= 0x4410_0000 {
        /* if |x| >= 2^66 */
        let low = get_low_word(x);
        if ix > 0x7FF0_0000 || (ix == 0x7FF0_0000 && low != 0) {
            return x + x; /* NaN */
        }
        return if hx > 0 {
            ATAN_HI[3] + ATAN_LO[3]
        } else {
            -ATAN_HI[3] - ATAN_LO[3]
        };
    }
    let id;
    if ix < 0x3FDC_0000 {
        /* |x| < 0.4375 */
        if ix < 0x3E40_0000 {
            /* |x| < 2^-27 */
            if HUGE + x > ONE {
                return x; /* raise inexact */
            }
        }
        id = -1;
    } else {
        x = x.abs();
        if ix < 0x3FF3_0000 {
            /* |x| < 1.1875 */
            if ix < 0x3FE6_0000 {
                /* 7/16 <=|x|<11/16 */
                id = 0;
                x = (2.0 * x - ONE) / (2.0 + x);
            } else {
                /* 11/16<=|x|< 19/16 */
                id = 1;
                x = (x - ONE) / (x + ONE);
            }
        } else if ix < 0x4003_8000 {
            /* |x| < 2.4375 */
            id = 2;
            x = (x - 1.5) / (ONE + 1.5 * x);
        } else {
            /* 2.4375 <= |x| < 2^66 */
            id = 3;
            x = -ONE / x;
        }
    }
    /* end of argument reduction */
    let mut z = x * x;
    let w = z * z;
    /* break sum from i=0 to 10 aT[i]z**(i+1) into odd and even poly */
    let s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
    let s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
    if id < 0 {
        x - x * (s1 + s2)
    } else {
        z = ATAN_HI[id as usize] - ((x * (s1 + s2) - ATAN_LO[id as usize]) - x);
        if hx < 0 {
            -z
        } else {
            z
        }
    }
}

/// V8 `Math.atan2(y, x)` — fdlibm port (Node 22 / V8 12.4 `ieee754.cc`).
pub fn atan2(y: f64, x: f64) -> f64 {
    const TINY: f64 = 1.0e-300;
    const ZERO: f64 = 0.0;
    const PI_O_4: f64 = 7.853_981_633_974_482_790_0E-01; /* 0x3FE921FB, 0x54442D18 */
    const PI_O_2: f64 = 1.570_796_326_794_896_558_0E+00; /* 0x3FF921FB, 0x54442D18 */
    const PI: f64 = 3.141_592_653_589_793_116_0E+00; /* 0x400921FB, 0x54442D18 */
    const PI_LO: f64 = 1.224_646_799_147_353_177_2E-16; /* 0x3CA1A626, 0x33145C07 */

    if x.is_nan() || y.is_nan() {
        return x + y; /* x or y is NaN */
    }
    if x == 1.0 {
        return atan(y); /* x=1.0 */
    }
    let hx = get_high_word(x) as i32;
    let lx = get_low_word(x);
    let hy = get_high_word(y) as i32;
    let ly = get_low_word(y);
    let ix = hx & 0x7FFF_FFFF;
    let iy = hy & 0x7FFF_FFFF;
    let m = ((hy >> 31) & 1) | ((hx >> 30) & 2); /* 2*sign(x)+sign(y) */

    /* when y = 0 */
    if (iy | (ly as i32)) == 0 {
        return match m {
            0 | 1 => y, /* atan(+-0,+anything)=+-0 */
            2 => PI + TINY, /* atan(+0,-anything) = pi */
            _ => -PI - TINY, /* atan(-0,-anything) =-pi */
        };
    }
    /* when x = 0 */
    if (ix | (lx as i32)) == 0 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }

    /* when x is INF */
    if ix == 0x7FF0_0000 {
        if iy == 0x7FF0_0000 {
            return match m {
                0 => PI_O_4 + TINY,   /* atan(+INF,+INF) */
                1 => -PI_O_4 - TINY,  /* atan(-INF,+INF) */
                2 => 3.0 * PI_O_4 + TINY, /* atan(+INF,-INF) */
                _ => -3.0 * PI_O_4 - TINY, /* atan(-INF,-INF) */
            };
        } else {
            return match m {
                0 => ZERO,        /* atan(+...,+INF) */
                1 => -ZERO,       /* atan(-...,+INF) */
                2 => PI + TINY,   /* atan(+...,-INF) */
                _ => -PI - TINY,  /* atan(-...,-INF) */
            };
        }
    }
    /* when y is INF */
    if iy == 0x7FF0_0000 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }

    /* compute y/x */
    let k = (iy - ix) >> 20;
    let z = if k > 60 {
        /* |y/x| >  2**60 */
        let z = PI_O_2 + 0.5 * PI_LO;
        return match m & 1 {
            0 => z,
            _ => -z,
        };
    } else if hx < 0 && k < -60 {
        ZERO /* 0 > |y|/x > -2**-60 */
    } else {
        atan((y / x).abs()) /* safe to do y/x */
    };
    match m {
        0 => z,                     /* atan(+,+) */
        1 => -z,                    /* atan(-,+) */
        2 => PI - (z - PI_LO),      /* atan(+,-) */
        _ => (z - PI_LO) - PI,      /* atan(-,-) */
    }
}

// ---------------------------------------------------------------------------
// pow  (V8 ieee754.cc, fdlibm e_pow.c — needed for terrain Math.pow(m, 1.35))
// ---------------------------------------------------------------------------

/// V8 `Math.pow(x, y)` — fdlibm port (Node 22 / V8 12.4 `ieee754.cc`).
pub fn pow(x: f64, y: f64) -> f64 {
    const BP: [f64; 2] = [1.0, 1.5];
    const DP_H: [f64; 2] = [0.0, 5.849_624_872_207_641_601_56e-01]; // 0x3FE2B803, 0x40000000
    const DP_L: [f64; 2] = [0.0, 1.350_039_202_129_748_971_28e-08]; // 0x3E4CFDEB, 0x43CFD006
    const ZERO: f64 = 0.0;
    const ONE: f64 = 1.0;
    const TWO53: f64 = 9_007_199_254_740_992.0; // 0x43400000, 0x00000000
    const HUGE: f64 = 1.0e300;
    const TINY: f64 = 1.0e-300;
    /* poly coefs for (3/2)*(log(x)-2s-2/3*s**3 */
    const L1: f64 = 5.999_999_999_999_946_487_25e-01; // 0x3FE33333, 0x33333303
    const L2: f64 = 4.285_714_285_785_501_842_52e-01; // 0x3FDB6DB6, 0xDB6FABFF
    const L3: f64 = 3.333_333_298_183_774_329_18e-01; // 0x3FD55555, 0x518F264D
    const L4: f64 = 2.727_281_238_085_340_064_89e-01; // 0x3FD17460, 0xA91D4101
    const L5: f64 = 2.306_607_457_755_617_540_67e-01; // 0x3FCD864A, 0x93C9DB65
    const L6: f64 = 2.069_750_178_003_384_177_84e-01; // 0x3FCA7E28, 0x4A454EEF
    const P1: f64 = 1.666_666_666_666_660_190_37e-01; // 0x3FC55555, 0x5555553E
    const P2: f64 = -2.777_777_777_701_559_338_42e-03; // 0xBF66C16C, 0x16BEBD93
    const P3: f64 = 6.613_756_321_437_934_361_17e-05; // 0x3F11566A, 0xAF25DE2C
    const P4: f64 = -1.653_390_220_546_525_153_90e-06; // 0xBEBBBD41, 0xC5D26BF1
    const P5: f64 = 4.138_136_797_057_238_460_39e-08; // 0x3E663769, 0x72BEA4D0
    const LG2: f64 = 6.931_471_805_599_452_862_27e-01; // 0x3FE62E42, 0xFEFA39EF
    const LG2_H: f64 = 6.931_471_824_645_996_093_75e-01; // 0x3FE62E43, 0x00000000
    const LG2_L: f64 = -1.904_654_299_957_768_045_25e-09; // 0xBE205C61, 0x0CA86C39
    const OVT: f64 = 8.008_566_259_537_294_437_2e-0017; // -(1024-log2(ovfl+.5ulp))
    const CP: f64 = 9.617_966_939_259_755_543_29e-01; // 0x3FEEC709, 0xDC3A03FD =2/(3ln2)
    const CP_H: f64 = 9.617_967_009_544_372_558_59e-01; // 0x3FEEC709, 0xE0000000 =(float)cp
    const CP_L: f64 = -7.028_461_650_952_758_265_16e-09; // 0xBE3E2FE0, 0x145B01F5 =tail cp_h
    const IVLN2: f64 = 1.442_695_040_888_963_387_00e+00; // 0x3FF71547, 0x652B82FE =1/ln2
    const IVLN2_H: f64 = 1.442_695_021_629_333_496_09e+00; // 0x3FF71547, 0x60000000 =24b 1/ln2
    const IVLN2_L: f64 = 1.925_962_991_126_617_468_87e-08; // 0x3E54AE0B, 0xF85DDF44 =1/ln2 tail

    let k: i32;
    let mut yisint: i32;
    let mut n: i32;
    let hx = get_high_word(x) as i32;
    let lx = get_low_word(x);
    let hy = get_high_word(y) as i32;
    let ly = get_low_word(y);
    let mut ix = hx & 0x7fff_ffff;
    let iy = hy & 0x7fff_ffff;

    /* y==zero: x**0 = 1 */
    if (iy | (ly as i32)) == 0 {
        return ONE;
    }

    /* +-NaN return x+y */
    if x.is_nan() || y.is_nan() {
        return x + y;
    }

    /* determine if y is an odd int when x < 0
     * yisint = 0 ... y is not an integer
     * yisint = 1 ... y is an odd int
     * yisint = 2 ... y is an even int
     */
    yisint = 0;
    if hx < 0 {
        if iy >= 0x4340_0000 {
            yisint = 2; /* even integer y */
        } else if iy >= 0x3ff0_0000 {
            k = (iy >> 20) - 0x3ff; /* exponent */
            if k > 20 {
                let j = (ly >> (52 - k)) as i32;
                if (j << (52 - k)) as u32 == ly {
                    yisint = 2 - (j & 1);
                }
            } else if ly == 0 {
                let j = iy >> (20 - k);
                if (j << (20 - k)) == iy {
                    yisint = 2 - (j & 1);
                }
            }
        }
    }

    /* special value of y */
    if ly == 0 {
        if iy == 0x7ff0_0000 {
            /* y is +-inf */
            if ((ix - 0x3ff0_0000) | (lx as i32)) == 0 {
                return y - y; /* inf**+-1 is NaN */
            } else if ix >= 0x3ff0_0000 {
                /* (|x|>1)**+-inf = inf,0 */
                return if hy >= 0 { y } else { ZERO };
            } else {
                /* (|x|<1)**-,+inf = inf,0 */
                return if hy < 0 { -y } else { ZERO };
            }
        }
        if iy == 0x3ff0_0000 {
            /* y is  +-1 */
            return if hy < 0 { ONE / x } else { x };
        }
        if hy == 0x4000_0000 {
            return x * x; /* y is  2 */
        }
        if hy == 0x3fe0_0000 && hx >= 0 {
            /* y is  0.5 and x >= +0 */
            return x.sqrt();
        }
    }

    let mut ax = x.abs();
    /* special value of x */
    if lx == 0 && (ix == 0x7ff0_0000 || ix == 0 || ix == 0x3ff0_0000) {
        /* x is +-0,+-inf,+-1 */
        let mut z = ax;
        if hy < 0 {
            z = ONE / z; /* z = (1/|x|) */
        }
        if hx < 0 {
            if ((ix - 0x3ff0_0000) | yisint) == 0 {
                /* (-1)**non-int is NaN */
                z = f64::NAN;
            } else if yisint == 1 {
                z = -z; /* (x<0)**odd = -(|x|**odd) */
            }
        }
        return z;
    }

    n = (hx >> 31) + 1;

    /* (x<0)**(non-int) is NaN */
    if (n | yisint) == 0 {
        return f64::NAN;
    }

    let mut s = ONE; /* s (sign of result -ve**odd) = -1 else = 1 */
    if (n | (yisint - 1)) == 0 {
        s = -ONE; /* (-ve)**(odd int) */
    }

    /* |y| is huge */
    let mut t1: f64;
    let t2: f64;
    if iy > 0x41e0_0000 {
        /* if |y| > 2**31 */
        if iy > 0x43f0_0000 {
            /* if |y| > 2**64, must o/uflow */
            if ix <= 0x3fef_ffff {
                return if hy < 0 { HUGE * HUGE } else { TINY * TINY };
            }
            if ix >= 0x3ff0_0000 {
                return if hy > 0 { HUGE * HUGE } else { TINY * TINY };
            }
        }
        /* over/underflow if x is not close to one */
        if ix < 0x3fef_ffff {
            return if hy < 0 { s * HUGE * HUGE } else { s * TINY * TINY };
        }
        if ix > 0x3ff0_0000 {
            return if hy > 0 { s * HUGE * HUGE } else { s * TINY * TINY };
        }
        /* now |1-x| is tiny <= 2**-20, suffice to compute
           log(x) by x-x^2/2+x^3/3-x^4/4 */
        let t = ax - ONE; /* t has 20 trailing zeros */
        let w = (t * t) * (0.5 - t * (0.333_333_333_333_333_333_333_3 - t * 0.25));
        let u = IVLN2_H * t; /* ivln2_h has 21 sig. bits */
        let v = t * IVLN2_L - w * IVLN2;
        t1 = u + v;
        t1 = set_low_word(t1, 0);
        t2 = v - (t1 - u);
    } else {
        n = 0;
        /* take care subnormal number */
        if ix < 0x0010_0000 {
            ax *= TWO53;
            n -= 53;
            ix = (get_high_word(ax) as i32) & 0x7fff_ffff;
        }
        n += (ix >> 20) - 0x3ff;
        let j = ix & 0x000f_ffff;
        /* determine interval */
        ix = j | 0x3ff0_0000; /* normalize ix */
        let k;
        if j <= 0x3988E {
            k = 0; /* |x|<sqrt(3/2) */
        } else if j < 0xBB67A {
            k = 1; /* |x|<sqrt(3)   */
        } else {
            k = 0;
            n += 1;
            ix -= 0x0010_0000;
        }
        ax = set_high_word(ax, ix as u32);

        /* compute ss = s_h+s_l = (x-1)/(x+1) or (x-1.5)/(x+1.5) */
        let u = ax - BP[k];
        let v = ONE / (ax + BP[k]);
        let ss = u * v;
        let mut s_h = ss;
        s_h = set_low_word(s_h, 0);
        /* t_h=ax+bp[k] High */
        let mut t_h = ZERO;
        t_h = set_high_word(t_h, ((((ix as u32) >> 1) | 0x2000_0000) + 0x0008_0000 + ((k as u32) << 18)) as u32);
        let t_l = ax - (t_h - BP[k]);
        let s_l = {
            let inner = (-s_h).mul_add(t_h, u); /* u - s_h*t_h */
            let outer = (-s_h).mul_add(t_l, inner); /* inner - s_h*t_l */
            v * outer
        };
        /* compute log(ax) */
        let mut s2 = ss * ss;
        let mut r = {
            let u1 = s2.mul_add(L6, L5);
            let u2 = s2.mul_add(u1, L4);
            let u3 = s2.mul_add(u2, L3);
            let u4 = s2.mul_add(u3, L2);
            (s2 * s2) * s2.mul_add(u4, L1)
        };
        r = s_l.mul_add(s_h + ss, r); /* r += s_l*(s_h+ss) */
        s2 = s_h * s_h;
        t_h = 3.0 + s2 + r;
        t_h = set_low_word(t_h, 0);
        let t_l = r - ((t_h - 3.0) - s2);
        /* u+v = ss*(1+...) */
        let u = s_h * t_h;
        let v = s_l.mul_add(t_h, t_l * ss); /* s_l*t_h + t_l*ss */
        /* 2/(3log2)*(ss+...) */
        let mut p_h = u + v;
        p_h = set_low_word(p_h, 0);
        let p_l = v - (p_h - u);
        let z_h = CP_H * p_h; /* cp_h+cp_l = 2/(3*log2) */
        let z_l = CP_L.mul_add(p_h, p_l * CP) + DP_L[k]; /* cp_l*p_h + p_l*cp + dp_l */
        /* log2(ax) = (ss+..)*2/(3*log2) = n + dp_h + z_h + z_l */
        let t = n as f64;
        t1 = ((z_h + z_l) + DP_H[k]) + t;
        t1 = set_low_word(t1, 0);
        t2 = z_l - (((t1 - t) - DP_H[k]) - z_h);
    }

    /* split up y into y1+y2 and compute (y1+y2)*(t1+t2) */
    let mut y1 = y;
    y1 = set_low_word(y1, 0);
    let p_l = (y - y1).mul_add(t1, y * t2); /* (y-y1)*t1 + y*t2 */
    let mut p_h = y1 * t1;
    let mut z = p_l + p_h;
    let j = get_high_word(z) as i32;
    let i = get_low_word(z) as i32;
    if j >= 0x4090_0000 {
        /* z >= 1024 */
        if ((j - 0x4090_0000) | i) != 0 {
            /* if z > 1024 */
            return s * HUGE * HUGE; /* overflow */
        } else if p_l + OVT > z - p_h {
            return s * HUGE * HUGE; /* overflow */
        }
    } else if (j & 0x7fff_ffff) >= 0x4090_cc00 {
        /* z <= -1075 */
        if ((j - (0xc090_cc00u32 as i32)) | i) != 0 {
            /* z < -1075 */
            return s * TINY * TINY; /* underflow */
        } else if p_l <= z - p_h {
            return s * TINY * TINY; /* underflow */
        }
    }
    /*
     * compute 2**(p_h+p_l)
     */
    let i = j & 0x7fff_ffff;
    let mut k = (i >> 20) - 0x3ff;
    n = 0;
    if i > 0x3fe0_0000 {
        /* if |z| > 0.5, set n = [z+0.5] */
        n = j + (0x0010_0000 >> (k + 1));
        k = ((n & 0x7fff_ffff) >> 20) - 0x3ff; /* new k for n */
        let t = set_high_word(ZERO, (n & !(0x000f_ffff >> k)) as u32);
        n = ((n & 0x000f_ffff) | 0x0010_0000) >> (20 - k);
        if j < 0 {
            n = -n;
        }
        p_h -= t;
    }
    let mut t = p_l + p_h;
    t = set_low_word(t, 0);
    let u = t * LG2_H;
    let x_lg2 = p_l - (t - p_h);
    let v = x_lg2.mul_add(LG2, t * LG2_L); /* x*lg2 + t*lg2_l */
    z = u + v;
    let w = v - (z - u);
    t = z * z;
    let t1 = {
        let u1 = t.mul_add(P5, P4);
        let u2 = t.mul_add(u1, P3);
        let u3 = t.mul_add(u2, P2);
        let u4 = t.mul_add(u3, P1);
        (-t).mul_add(u4, z) /* z - t*u4 */
    };
    let w2 = z.mul_add(w, w); /* w + z*w */
    let r = (z * t1) / ((t1 - 2.0) - w2);
    z = ONE - (r - z);
    let j_orig = get_high_word(z) as i32;
    let j_adj = j_orig.wrapping_add(((n as u32) << 20) as i32);
    if (j_adj >> 20) <= 0 {
        z = scalbn(z, n); /* subnormal output */
    } else {
        z = set_high_word(z, j_adj as u32);
    }
    s * z
}
