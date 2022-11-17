//! This score is used to resolve conflicts when we have multiple values with the same number of tracks.
//! Thus, more trending tracks will carry more weight.

const TRACK_COUNT: usize = 100;
const SCORE_COEFFICIENT: f32 = 0.001; // Some small random value

pub fn get_score(track_position: usize) -> f32 {
    1.0 + (-2.0 * SCORE_COEFFICIENT * (track_position as f32) / ((TRACK_COUNT as f32) - 1.0))
        + SCORE_COEFFICIENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_calculation() {
        assert!(get_score(0) > get_score(TRACK_COUNT - 1));
    }

    #[test]
    fn test_score_sum() {
        assert!(
            (TRACK_COUNT as f32 - (0..TRACK_COUNT).map(get_score).sum::<f32>().abs())
                < f32::EPSILON
        );
    }
}
