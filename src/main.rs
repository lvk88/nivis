use mywasm::Simulation;

fn main() {
    let mut s = Simulation::new(300, 600);
    for i in 0..500 {
        s.step();
    }
}
