//! Odds engine: closed-form 5-star probabilities from the soft-pity curve, plus a seeded Monte Carlo
//! used for the luck percentile chart.

use crate::pity::PityRules;
use serde::Serialize;

/// Probability that pull number `k` (1-based, counted since the last 5-star) is a 5-star.
pub fn rate_at(r: &PityRules, k: u32) -> f64 {
    if k >= r.hard_pity {
        return 1.0;
    }
    if k < r.soft_start {
        return r.base_rate;
    }
    (r.base_rate + r.soft_step * (k - r.soft_start + 1) as f64).min(1.0)
}

/// Distribution of the pull number of the next 5-star, given `pity` pulls already made without one.
/// Index `i` holds P(next 5-star lands exactly `i + 1` pulls from now).
pub fn pmf_from(r: &PityRules, pity: u32) -> Vec<f64> {
    let pity = pity.min(r.hard_pity - 1);
    let mut survive = 1.0;
    let mut out = Vec::with_capacity((r.hard_pity - pity) as usize);
    for k in (pity + 1)..=r.hard_pity {
        let p = rate_at(r, k);
        out.push(survive * p);
        survive *= 1.0 - p;
    }
    out
}

/// Distribution of the pity value at which a 5-star is hit, starting from zero.
pub fn pmf(r: &PityRules) -> Vec<f64> {
    pmf_from(r, 0)
}

fn cdf(pmf: &[f64], n: u32) -> f64 {
    pmf.iter().take(n as usize).sum::<f64>().min(1.0)
}

/// P(at least one 5-star within the next `n` pulls).
pub fn p_within(r: &PityRules, pity: u32, n: u32) -> f64 {
    cdf(&pmf_from(r, pity), n)
}

/// P(the featured 5-star within the next `n` pulls), honouring the 50/50 (or 75/25) and guarantee.
///
/// If not guaranteed: P = q * F(n) + (1 - q) * sum_j f(j) * G(n - j)
/// where `f`/`F` is the first-hit distribution from the current pity and `G` the fresh-start CDF.
pub fn p_featured_within(r: &PityRules, pity: u32, guaranteed: bool, n: u32) -> f64 {
    let f = pmf_from(r, pity);
    let first = cdf(&f, n);
    if guaranteed || !r.has_fifty_fifty() {
        return first;
    }
    let q = r.featured_rate;
    if !r.carries_guarantee {
        // Every 5-star is an independent q-roll: sum over the number of 5-stars is overkill for the UI,
        // so approximate with "first or second hit".
        let g = pmf(r);
        let second: f64 = f.iter().enumerate().take(n as usize).map(|(j, fj)| fj * cdf(&g, n - j as u32 - 1)).sum();
        return q * first + (1.0 - q) * q * second;
    }
    let g = pmf(r);
    let after_loss: f64 = f.iter().enumerate().take(n as usize).map(|(j, fj)| fj * cdf(&g, n - j as u32 - 1)).sum();
    (q * first + (1.0 - q) * after_loss).min(1.0)
}

/// Mean pity of a 5-star from a fresh start.
pub fn expected_pity(r: &PityRules) -> f64 {
    pmf(r).iter().enumerate().map(|(i, p)| (i + 1) as f64 * p).sum()
}

/// Smallest `n` such that P(featured within n) >= `target`.
pub fn pulls_for_confidence(r: &PityRules, pity: u32, guaranteed: bool, target: f64) -> u32 {
    let max = r.hard_pity * 2;
    (1..=max).find(|&n| p_featured_within(r, pity, guaranteed, n) >= target).unwrap_or(max)
}

#[derive(Debug, Clone, Serialize)]
pub struct CurvePoint {
    pub n: u32,
    pub any: f64,
    pub featured: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OddsReport {
    pub pity: u32,
    pub guaranteed: bool,
    pub pulls: u32,
    pub p_any: f64,
    pub p_featured: f64,
    pub expected_pity: f64,
    pub pulls_for_50: u32,
    pub pulls_for_90: u32,
    pub curve: Vec<CurvePoint>,
    pub rate_curve: Vec<f64>,
}

pub fn report(r: &PityRules, pity: u32, guaranteed: bool, pulls: u32) -> OddsReport {
    let max = r.hard_pity * 2;
    let curve = (0..=max)
        .step_by(2)
        .map(|n| CurvePoint { n, any: p_within(r, pity, n), featured: p_featured_within(r, pity, guaranteed, n) })
        .collect();
    OddsReport {
        pity,
        guaranteed,
        pulls,
        p_any: p_within(r, pity, pulls),
        p_featured: p_featured_within(r, pity, guaranteed, pulls),
        expected_pity: expected_pity(r),
        pulls_for_50: pulls_for_confidence(r, pity, guaranteed, 0.5),
        pulls_for_90: pulls_for_confidence(r, pity, guaranteed, 0.9),
        curve,
        rate_curve: (1..=r.hard_pity).map(|k| rate_at(r, k)).collect(),
    }
}

/// xorshift64* - tiny, seeded, good enough for UI simulations and reproducible tests.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n.max(1) as u64) as usize
    }
}

/// Draws one 5-star pity value from the soft-pity curve.
pub fn sample_pity(r: &PityRules, rng: &mut Rng) -> u32 {
    for k in 1..=r.hard_pity {
        if rng.next_f64() < rate_at(r, k) {
            return k;
        }
    }
    r.hard_pity
}

#[derive(Debug, Clone, Serialize)]
pub struct LuckReport {
    /// Observed average pity per 5-star.
    pub observed: f64,
    pub expected: f64,
    /// Share of simulated players with a worse (higher) average pity, 0..100.
    pub percentile: f64,
    pub samples: u32,
    /// Histogram of simulated average pity, bins of `bin_width` starting at `bin_start`.
    pub bin_start: u32,
    pub bin_width: u32,
    pub bins: Vec<u32>,
}

/// Compares an observed average pity over `count` 5-stars with `trials` simulated players.
pub fn luck_percentile(r: &PityRules, observed: f64, count: u32, trials: u32, seed: u64) -> LuckReport {
    let mut rng = Rng::new(seed);
    let count = count.max(1);
    let (bin_start, bin_width) = (r.hard_pity / 3, 2u32);
    let mut bins = vec![0u32; ((r.hard_pity - bin_start) / bin_width + 1) as usize];
    let mut worse = 0u32;
    for _ in 0..trials {
        let sum: u32 = (0..count).map(|_| sample_pity(r, &mut rng)).sum();
        let mean = sum as f64 / count as f64;
        if mean > observed {
            worse += 1;
        } else if (mean - observed).abs() < 1e-9 {
            worse += rng.below(2) as u32; // split ties
        }
        let b = ((mean.max(bin_start as f64) - bin_start as f64) / bin_width as f64) as usize;
        let last = bins.len() - 1;
        bins[b.min(last)] += 1;
    }
    LuckReport {
        observed,
        expected: expected_pity(r),
        percentile: if trials == 0 { 50.0 } else { worse as f64 * 100.0 / trials as f64 },
        samples: trials,
        bin_start,
        bin_width,
        bins,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BannerKind, Game};

    fn genshin() -> PityRules {
        PityRules::for_banner(Game::Genshin, BannerKind::Character)
    }

    #[test]
    fn rate_curve_shape() {
        let r = genshin();
        assert_eq!(rate_at(&r, 1), 0.006);
        assert_eq!(rate_at(&r, 73), 0.006);
        assert!((rate_at(&r, 74) - 0.066).abs() < 1e-12);
        assert_eq!(rate_at(&r, 90), 1.0);
    }

    #[test]
    fn pmf_sums_to_one_for_every_banner() {
        for g in Game::ALL {
            for b in g.banners() {
                let r = PityRules::for_banner(g, *b);
                for pity in [0, 30, 70, r.hard_pity - 1] {
                    let s: f64 = pmf_from(&r, pity).iter().sum();
                    assert!((s - 1.0).abs() < 1e-9, "{g:?} {b:?} pity {pity}: {s}");
                }
            }
        }
    }

    #[test]
    fn expected_pity_matches_community_value() {
        // Widely cited consolidated rate for the Genshin character banner is 1.6% => ~62.3 pulls.
        let e = expected_pity(&genshin());
        assert!((61.5..63.5).contains(&e), "expected pity {e}");
    }

    #[test]
    fn within_hard_pity_is_certain() {
        let r = genshin();
        assert!((p_within(&r, 0, 90) - 1.0).abs() < 1e-9);
        assert!((p_within(&r, 80, 10) - 1.0).abs() < 1e-9);
        assert_eq!(p_within(&r, 10, 0), 0.0);
    }

    #[test]
    fn probabilities_are_monotone() {
        let r = genshin();
        let mut last = 0.0;
        for n in 0..=180 {
            let p = p_featured_within(&r, 20, false, n);
            assert!(p + 1e-12 >= last);
            assert!(p <= 1.0 + 1e-12);
            last = p;
        }
        assert!((p_featured_within(&r, 0, false, 180) - 1.0).abs() < 1e-9, "180 pulls is the full guarantee");
    }

    #[test]
    fn guarantee_beats_fifty_fifty() {
        let r = genshin();
        let g = p_featured_within(&r, 40, true, 50);
        let f = p_featured_within(&r, 40, false, 50);
        assert!(g > f);
        assert!((g - p_within(&r, 40, 50)).abs() < 1e-12);
    }

    #[test]
    fn confidence_targets() {
        let r = genshin();
        let n50 = pulls_for_confidence(&r, 0, false, 0.5);
        let n90 = pulls_for_confidence(&r, 0, false, 0.9);
        assert!(n50 < n90 && n90 <= 180);
        assert!(pulls_for_confidence(&r, 0, true, 0.99) <= 90);
    }

    #[test]
    fn simulation_is_seeded_and_sane() {
        let r = genshin();
        let a = luck_percentile(&r, 55.0, 12, 4000, 7);
        let b = luck_percentile(&r, 55.0, 12, 4000, 7);
        assert_eq!(a.percentile, b.percentile);
        assert!(a.percentile > 60.0, "avg 55 over 12 five-stars is lucky: {}", a.percentile);
        assert_eq!(a.bins.iter().sum::<u32>(), 4000);
        let unlucky = luck_percentile(&r, 76.0, 12, 4000, 7);
        assert!(unlucky.percentile < 10.0);
    }

    #[test]
    fn sampled_mean_close_to_closed_form() {
        let r = genshin();
        let mut rng = Rng::new(42);
        let n = 20_000;
        let mean = (0..n).map(|_| sample_pity(&r, &mut rng) as f64).sum::<f64>() / n as f64;
        assert!((mean - expected_pity(&r)).abs() < 0.6, "{mean}");
    }
}
