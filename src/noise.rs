use clap::ValueEnum;
use noise::{Fbm, NoiseFn, Perlin, Simplex};

/// TODO: deprecate this in favor of configured values

/// NOTE: update this whenever number of selectors changes
pub const NOISE_SELECTORS_LEN: isize = 3;

/// A flat enum for selecting noise functions as a CLI option or config variable.
#[derive(Debug, Clone, Copy, ValueEnum)]
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

    pub fn get_next(&self) -> Self {
        Self::from((*self as isize + 1) % NOISE_SELECTORS_LEN)
    }

    pub fn get_prev(&self) -> Self {
        Self::from((*self as isize + NOISE_SELECTORS_LEN - 1) % NOISE_SELECTORS_LEN)
    }
}

impl From<isize> for NoiseSelector {
    fn from(idx: isize) -> Self {
        match idx {
            0 => Self::Simplex,
            1 => Self::Perlin,
            2 => Self::FbmPerlin,
            _ => Self::default(),
        }
    }
}

impl Default for NoiseSelector {
    fn default() -> Self {
        Self::Perlin
    }
}

#[cfg(test)]
mod tests {
    use crate::noise::{NoiseSelector, NOISE_SELECTORS_LEN};

    #[test]
    fn test_noise_selector_from_idx() {
        for i in 0..NOISE_SELECTORS_LEN {
            assert_eq!(i, NoiseSelector::from(i) as isize)
        }
        // for larger numbers, should be equal to default
        assert_eq!(
            NoiseSelector::default() as isize,
            NoiseSelector::from(255) as isize
        );
    }
}
