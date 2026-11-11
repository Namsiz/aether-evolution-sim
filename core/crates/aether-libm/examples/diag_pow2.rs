fn main() { let x = 0.07848557480610907f64; let y = 1.0 + x*0.7; let r = aether_libm::pow(x, y);
  println!("RS  pow({x:e}, {y:e}) = 0x{:016x}", r.to_bits()); }
