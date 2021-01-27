use clap::Clap;

/// Some extra description here...
#[derive(Debug, Clap)]
#[clap(
    name = "Genetic Algorithms",
    version = "0.1",
    author = "Renato H. <renato.hermoza@pucp.edu.pe>"
)]
pub struct Opts {
    /// Number of generations to run the optimization
    #[clap(short, long, default_value = "1000")]
    pub generations: u64,
    /// Population size (number of individuals)
    #[clap(short, long, default_value = "20")]
    pub pop_sz: usize,
    /// Lower limit for random generation of individual positions
    #[clap(
        short,
        long,
        default_value = "-5",
    )]
    pub lower_limit: f32,
    /// Upper limit for random generation of individual positions
    #[clap(
        short,
        long,
        default_value = "5",
    )]
    pub upper_limit: f32,
}
