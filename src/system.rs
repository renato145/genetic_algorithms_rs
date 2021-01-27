use crate::Opts;
use console::style;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use ndarray::Array1;
use ndarray_rand::{rand_distr::Uniform, RandomExt};
use specs::{prelude::*, Component};

#[derive(Default)]
struct PopulationSize(usize);

#[derive(Default)]
struct NDimensions(usize);

struct RandomDistr(Uniform<f32>);

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Position(Array1<f32>);

impl Position {
    fn from_distr(ndim: usize, range: &Uniform<f32>) -> Position {
        Position(Array1::random(ndim, range))
    }
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Fitness(f32);

#[derive(Default)]
struct InputTest(Array1<f32>);

struct FitnessCalculator;
impl<'a> System<'a> for FitnessCalculator {
    type SystemData = (
        Entities<'a>,
        Read<'a, InputTest>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Fitness>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (entities, input, position_storage, mut fitness_storage, updater): Self::SystemData,
    ) {
        (
            &*entities,
            &position_storage,
            (&mut fitness_storage).maybe(),
        )
            .par_join()
            .for_each(|(entity, position, fitness)| {
                let value = input.0.clone() * position.0.clone();
                let value = value.sum();
                match fitness {
                    Some(fitness) => fitness.0 = value,
                    None => updater.insert(entity, Fitness(value)),
                };
            });
    }
}

pub fn run_world(opts: Opts, inputs: Array1<f32>) {
    let ndim = inputs.len();

    let mut world = World::new();
    world.insert(PopulationSize(opts.pop_sz));
    world.insert(NDimensions(ndim));
    world.insert(RandomDistr(Uniform::new_inclusive(
        opts.lower_limit,
        opts.upper_limit,
    )));
    world.insert(InputTest(inputs));

    let mut dispatcher = DispatcherBuilder::new()
        .with(FitnessCalculator, "fitness_calc", &[])
        .build();

    dispatcher.setup(&mut world);

    // Create population
    {
        let entities: Vec<_> = world.create_iter().take(opts.pop_sz).collect();
        let range = world.read_resource::<RandomDistr>().0;
        let mut positions = world.write_storage();
        for e in entities.iter() {
            positions
                .insert(*e, Position::from_distr(ndim, &range))
                .unwrap();
        }
    }
    world.maintain();

    println!(
        "{} {} individuals with {} dimensions",
        style("Population started:").bold(),
        opts.pop_sz,
        ndim
    );

    // Optimization loop
    let pb = ProgressBar::new(opts.generations).with_style(ProgressStyle::default_bar().template(
        "{spinner} [{elapsed_precise} - {per_sec}] [{wide_bar}] {msg} {pos}/{len} ({eta})",
    ));
    for _gen in (0..opts.generations).progress_with(pb) {
        dispatcher.dispatch(&world);
        world.maintain();
    }

    let _pos = world.read_storage::<Position>();
    let fit = world.read_storage::<Fitness>();
    // for ent in world.entities().join() {
    for ent in world.entities().join().take(2) {
        println!("{:?} - {:?}", ent, fit.get(ent));
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
