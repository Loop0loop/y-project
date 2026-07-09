pub(crate) fn ease_out(t: f32) -> f32 {
    cubic_bezier(t, 0.22, 1.0, 0.36, 1.0)
}

fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 || t == 1.0 {
        return t;
    }
    let mut lo = 0.0;
    let mut hi = 1.0;

    for _ in 0..10 {
        let mid = (lo + hi) * 0.5;
        if bezier(mid, x1, x2) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    bezier((lo + hi) * 0.5, y1, y2)
}

fn bezier(t: f32, a: f32, b: f32) -> f32 {
    let inv = 1.0 - t;
    3.0 * inv * inv * t * a + 3.0 * inv * t * t * b + t * t * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_out_keeps_endpoints() {
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
    }

    #[test]
    fn ease_out_moves_quickly_then_settles() {
        assert!(ease_out(0.25) > 0.5);
        assert!(ease_out(0.75) > 0.9);
    }
}
