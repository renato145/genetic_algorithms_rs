use crate::Opts;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::izip;
use rand::{distributions::Uniform, prelude::IteratorRandom};
use rand::{thread_rng, Rng};
use specs::{prelude::*, Component};

#[derive(Default)]
struct PopulationSize(usize);

#[derive(Default)]
struct NDimensions(usize);

#[derive(Default)]
struct Limits(f32, f32);

struct RandomDistr(Uniform<f32>);

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Position(Vec<f32>);

impl Position {
    fn from_distr(ndim: usize, range: &Uniform<f32>) -> Position {
        let mut rng = thread_rng();
        Position((&mut rng).sample_iter(range).take(ndim).collect())
    }
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Fitness(f32);

#[derive(Default)]
struct InputTest(Vec<f32>);

/// Calculate fitness logic, for this example this is a simple sum(multiplication(...))
fn eval_fitness(input: &Vec<f32>, weights: &Vec<f32>) -> f32 {
    input.iter().zip(weights).map(|(a, b)| a * b).sum()
}

struct FitnessCalc;
impl<'a> System<'a> for FitnessCalc {
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
                let value = eval_fitness(&input.0, &position.0);
                match fitness {
                    Some(fitness) => fitness.0 = value,
                    None => updater.insert(entity, Fitness(value)),
                };
            });
    }
}

#[derive(Default)]
struct MutationRatio(f32);

#[derive(Default)]
struct MutationFactor(f32);

/// Takes care of the evolution of individuals, for each individual it will:
/// 1. Pick random dimensions D based on a `MutationRatio`.
/// 2. Pick 3 other individuals (a, b and c).
/// 3. individual[D] = a[D] + F*(b[D] - c[D]), where F is a `MutationFactor`.
/// 4. Make sure individuals stay on the defined lower and upper limit.
/// 5. Keep change if fitness is better.
struct EvolveMechanism;
impl<'a> System<'a> for EvolveMechanism {
    type SystemData = (
        Read<'a, NDimensions>,
        Read<'a, MutationRatio>,
        Read<'a, MutationFactor>,
        Read<'a, InputTest>,
        Read<'a, Limits>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Fitness>,
    );

    fn run(
        &mut self,
        (
            ndim,
            mutation_ratio,
            mutation_factor,
            input,
            limits,
            mut position_storage,
            mut fitness_storage,
        ): Self::SystemData,
    ) {
        let mut rng = thread_rng();
        let range_dims = Uniform::new(0.0f32, 1.0f32);
        let (mut positions, all_fitness): (Vec<_>, Vec<_>) =
            (&mut position_storage, &mut fitness_storage).join().unzip();
        let pop_sz = positions.len();

        // Step 1
        let random_dims = (0..pop_sz)
            .map(|_| {
                (&mut rng)
                    .sample_iter(range_dims)
                    .enumerate()
                    .take(ndim.0)
                    .filter_map(|(i, n)| {
                        if &n < &mutation_ratio.0 {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // Step 2
        let other_individuals = (0..pop_sz)
            .map(|idx| {
                (0..pop_sz)
                    .filter(|i| i != &idx)
                    .choose_multiple(&mut rng, 3)
            })
            .collect::<Vec<_>>();

        for (idx, (dims, others, fitness)) in
            izip!(random_dims, other_individuals, all_fitness).enumerate()
        {
            let mut new_position = positions[idx].0.clone();
            for dim in dims {
                // Step 3
                let v = positions[others[0]].0[dim]
                    + mutation_factor.0
                        * (positions[others[1]].0[dim] - positions[others[2]].0[dim]);
                // Step 4
                new_position[dim] = v.max(limits.0).min(limits.1);
            }
            // Step 5
            let new_fitness = eval_fitness(&input.0, &new_position);
            if new_fitness < fitness.0 {
                positions[idx].0 = new_position;
                fitness.0 = new_fitness;
            }
        }
    }
}

#[derive(Default)]
struct BestFitness(Option<f32>);

struct BestFitnessCalc;
impl<'a> System<'a> for BestFitnessCalc {
    type SystemData = (ReadStorage<'a, Fitness>, Write<'a, BestFitness>);

    fn run(&mut self, (fitness_storage, mut best_fitness): Self::SystemData) {
        let min_fitness = (&fitness_storage)
            .join()
            .map(|o| o.0)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less));
        if let Some(fitness) = min_fitness {
            best_fitness.0 = Some(fitness);
        }
    }
}

struct ShowPositions;
impl<'a> System<'a> for ShowPositions {
    type SystemData = ReadStorage<'a, Position>;

    fn run(&mut self, position_storage: Self::SystemData) {
        let positions = (&position_storage).join().collect::<Vec<_>>();
        println!("{:?}", positions);
    }
}

pub fn run_world(opts: Opts, inputs: Vec<f32>) {
    let ndim = inputs.len();

    let mut world = World::new();
    world.insert(PopulationSize(opts.pop_sz));
    world.insert(NDimensions(ndim));
    world.insert(Limits(opts.lower_limit, opts.upper_limit));
    world.insert(RandomDistr(Uniform::new_inclusive(
        opts.lower_limit,
        opts.upper_limit,
    )));
    world.insert(InputTest(inputs));
    world.insert(MutationRatio(opts.mutation_ratio));
    world.insert(MutationFactor(opts.mutation_factor));

    let mut eval_fitness_dispatcher = DispatcherBuilder::new()
        .with(FitnessCalc, "fitness_calc", &[])
        .build();

    let mut dispatcher = DispatcherBuilder::new();

    if opts.verbose > 1 {
        dispatcher = dispatcher
            .with(ShowPositions, "before_evolve", &[])
            .with(EvolveMechanism, "evolve_mech", &["before_evolve"])
            .with(BestFitnessCalc, "best_fitness", &["evolve_mech"])
            .with(ShowPositions, "after_evolve", &["evolve_mech"]);
    } else {
        dispatcher = dispatcher.with(EvolveMechanism, "evolve_mech", &[]).with(
            BestFitnessCalc,
            "best_fitness",
            &["evolve_mech"],
        );
    }

    let mut dispatcher = dispatcher.build();

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

    // Get initial fitness
    eval_fitness_dispatcher.dispatch(&mut world);

    // Optimization loop
    let pb = ProgressBar::new(opts.generations).with_style(ProgressStyle::default_bar().template(
        "{spinner} [{elapsed_precise} - {per_sec}] [{wide_bar}] {msg} | {pos}/{len} ({eta})",
    ));
    for _gen in 0..opts.generations {
        dispatcher.dispatch(&mut world);
        world.maintain();
        let best_fitness = world.read_resource::<BestFitness>().0;
        pb.set_message(&format!(
            "{} {:.2}",
            style("Best fitness:").bold(),
            style(best_fitness.unwrap_or(99999.0)).bold()
        ));
        pb.inc(1);
    }
    pb.finish();

    let _pos = world.read_storage::<Position>();
    let fit = world.read_storage::<Fitness>();

    if opts.verbose > 0 {
        for ent in world.entities().join() {
            println!("{:?} - {:?}", ent, fit.get(ent));
        }
    }
}
