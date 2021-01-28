use genetic_algorithms::*;

fn main() {
    let opts = Opts::parse();
    let inputs = vec![4.0, -2.0, 3.5, 5.0, -11.0, -4.7, -9.0, 100.0, -23.0];

    run_world(opts, inputs);
}
