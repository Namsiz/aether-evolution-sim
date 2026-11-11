fn main() {
    let x = 0.5401962210889906f64;
    let y = 1.3781373547622935f64;
    let r = aether_libm::pow(x, y);
    println!("lib pow = 0x{:016x}", r.to_bits());
}
