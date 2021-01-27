use genetic_algorithms::*;
use ndarray::arr1;

fn main() {
    let opts = Opts::parse();
    let inputs = arr1(&[4.0, -2.0, 3.5, 5.0, -11.0, -4.7]);

    run_world(opts, inputs);
}
