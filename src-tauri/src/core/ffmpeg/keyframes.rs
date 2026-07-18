pub fn is_keyframe(timestamp: f64, keyframes: &[f64]) -> bool {
    const TOLERANCE: f64 = 0.001;
    keyframes
        .iter()
        .any(|&kf| (kf - timestamp).abs() < TOLERANCE)
}

pub fn find_prev_keyframe(timestamp: f64, keyframes: &[f64]) -> Option<f64> {
    let idx = match keyframes.binary_search_by(|&kf| {
        kf.partial_cmp(&timestamp)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        Ok(i) => i,
        Err(i) => i,
    };
    idx.checked_sub(1).and_then(|i| keyframes.get(i)).copied()
}

pub fn find_next_keyframe(timestamp: f64, keyframes: &[f64]) -> Option<f64> {
    let idx = match keyframes.binary_search_by(|&kf| {
        kf.partial_cmp(&timestamp)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        Ok(i) => i + 1,
        Err(i) => i,
    };
    keyframes.get(idx).copied()
}
