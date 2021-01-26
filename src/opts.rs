use clap::Clap;

/// Some extra description here...
#[derive(Debug, Clap)]
#[clap(
    name = "Genetic Algorithms",
    version = "0.1",
    author = "Renato H. <renato.hermoza@pucp.edu.pe>"
)]
pub struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "20")]
    pub population_size: usize,
    #[clap(short, long, default_value = "10")]
    pub num_dimensions: usize,
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
