use mywasm::Simulation;

use std::time::Instant;

fn main() {
    let mut s = Simulation::new(300, 600);

    let n_steps = 500;

    let tic = Instant::now();
    for _ in 0..500 {
        s.step();
        let _phi_rgb = s.get_phi_rgb();
    }
    let toc = Instant::now();

    let elapsed = toc - tic;
    let average_step_cost = elapsed.as_millis() as f64 / n_steps as f64;
    println!("Average step cost: {}", average_step_cost);
}
