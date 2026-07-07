pub fn is_keyframe(timestamp: f64, keyframes: &[f64]) -> bool {
    const TOLERANCE: f64 = 0.001;
    keyframes
        .iter()
        .any(|&kf| (kf - timestamp).abs() < TOLERANCE)
}

pub fn find_prev_keyframe(timestamp: f64, keyframes: &[f64]) -> Option<f64> {
    keyframes
        .iter()
        .filter(|&&kf| kf < timestamp)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .copied()
}

pub fn find_next_keyframe(timestamp: f64, keyframes: &[f64]) -> Option<f64> {
    keyframes
        .iter()
        .filter(|&&kf| kf > timestamp)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .copied()
}
