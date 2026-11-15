//! Deterministic pseudo-random numbers (xoshiro256++), implemented in-crate so
//! that a given seed reproduces the same game on every platform and toolchain.

#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
    spare: Option<f64>,
}

#[inline]
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut st = seed;
        let s = [splitmix64(&mut st), splitmix64(&mut st), splitmix64(&mut st), splitmix64(&mut st)];
        Rng { s, spare: None }
    }

    /// Independent child generator (for parallel games).
    pub fn fork(&mut self) -> Rng {
        Rng::new(self.next_u64())
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let result = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniform in [0, 1).
    #[inline]
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    #[inline]
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f64()
    }

    /// Uniform integer in 0..n.
    #[inline]
    pub fn below(&mut self, n: usize) -> usize {
        ((self.next_u64() >> 32) as usize * n) >> 32
    }

    #[inline]
    pub fn chance(&mut self, p: f64) -> bool {
        self.f64() < p
    }

    /// Standard normal variate (Marsaglia polar method).
    pub fn normal(&mut self) -> f64 {
        if let Some(z) = self.spare.take() {
            return z;
        }
        loop {
            let u = 2.0 * self.f64() - 1.0;
            let v = 2.0 * self.f64() - 1.0;
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                let f = (-2.0 * s.ln() / s).sqrt();
                self.spare = Some(v * f);
                return u * f;
            }
        }
    }

    #[inline]
    pub fn gauss(&mut self, mean: f64, sd: f64) -> f64 {
        mean + sd * self.normal()
    }

    /// Normal variate clamped to `[lo, hi]`.
    #[inline]
    pub fn gauss_clamped(&mut self, mean: f64, sd: f64, lo: f64, hi: f64) -> f64 {
        self.gauss(mean, sd).clamp(lo, hi)
    }

    /// Index drawn with probability proportional to `weights`.
    pub fn pick_weighted(&mut self, weights: &[f64]) -> usize {
        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            return 0;
        }
        let mut x = self.f64() * total;
        for (i, &w) in weights.iter().enumerate() {
            if x < w {
                return i;
            }
            x -= w;
        }
        weights.len() - 1
    }
}

/// Logistic function.
#[inline]
pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_well_distributed() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut r = Rng::new(7);
        let n = 200_000;
        let (mut sum, mut sq, mut u) = (0.0, 0.0, 0.0);
        for _ in 0..n {
            let z = r.normal();
            sum += z;
            sq += z * z;
            u += r.f64();
        }
        let mean = sum / n as f64;
        let var = sq / n as f64 - mean * mean;
        assert!(mean.abs() < 0.01 && (var - 1.0).abs() < 0.02, "mean {mean} var {var}");
        assert!((u / n as f64 - 0.5).abs() < 0.005);
    }

    #[test]
    fn weighted_pick_and_below() {
        let mut r = Rng::new(1);
        let mut c = [0usize; 3];
        for _ in 0..30_000 {
            c[r.pick_weighted(&[1.0, 2.0, 7.0])] += 1;
        }
        assert!((c[2] as f64 / 30_000.0 - 0.7).abs() < 0.02);
        for _ in 0..1000 {
            assert!(r.below(9) < 9);
        }
    }
}
