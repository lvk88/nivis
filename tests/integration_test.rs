use nivis::Simulation;
use assert_float_eq::*;

#[test]
fn run_one_step_simulation() {
    let mut s = Simulation::new(300, 300);
    s.add_seed(150, 150);

    s.step();

    // These are just some reference values I had from a test run
    assert_f32_near!(s.get_temperature(148, 150), -0.06765032);
    assert_f32_near!(s.get_temperature(149, 150), 0.0);

    assert_f32_near!(s.get_phi(148, 150), 0.95771855);
    assert_f32_near!(s.get_phi(149, 150), 1.0);
}
