use crate::Opts;
use specs::{Component, prelude::*};

struct PopulationSize(usize);
struct NDimensions(usize);
struct Limits(f32,f32);

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Position(Vec<f32>);

impl Position {
    fn new_random(n: usize, low: f32, high: f32) -> Position {
        Position(Vec::with_capacity(n))
    }
}

pub fn create_world(opts: Opts) -> World {
    let mut world = World::new();
    world.insert(PopulationSize(opts.population_size));
    world.insert(NDimensions(opts.num_dimensions));
    world.insert(Limits(opts.lower_limit, opts.upper_limit));
    // let t = Position::new_random(opts.dimensions);
    // dbg!(t);

    world
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
