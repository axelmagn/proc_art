use clap::ValueEnum;
use noise::{Fbm, NoiseFn, Perlin, Simplex};

/// A flat enum for selecting noise functions as a CLI option or config variable.
#[derive(Debug, Clone, ValueEnum)]
pub enum NoiseSelector {
    Simplex,
    Perlin,
    FbmPerlin,
}

impl NoiseSelector {
    pub fn get_noise_2d(&self, seed: u32) -> Box<dyn NoiseFn<f64, 2>> {
        match self {
            Self::Simplex => Box::new(Simplex::new(seed)),
            Self::Perlin => Box::new(Perlin::new(seed)),
            Self::FbmPerlin => Box::new(Fbm::<Perlin>::new(seed)),
        }
    }
}
