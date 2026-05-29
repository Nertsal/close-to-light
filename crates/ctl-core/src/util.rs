use geng::prelude::Float;

pub fn calculate_hash(bytes: &[u8]) -> String {
    use data_encoding::HEXLOWER;
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    HEXLOWER.encode(hasher.finalize().as_ref())
}

pub fn smoothstep<T: Float>(t: T) -> T {
    T::from_f32(3.0) * t * t - T::from_f32(2.0) * t * t * t
}

/// Finds the closest fraction to `val` where the denominator is <= `max_denom`.
pub fn limit_denominator(val: f64, max_denom: u64) -> (u64, u64) {
    if val == 0.0 {
        return (0, 1);
    }

    let mut m00 = 1;
    let mut m01 = 0;
    let mut m10 = 0;
    let mut m11 = 1;

    let mut x = val;

    loop {
        let a = x.floor() as u64;
        let next_x = x - a as f64;

        let d2 = m10 * a + m11;
        if d2 > max_denom {
            break;
        }

        let n2 = m00 * a + m01;
        m01 = m00;
        m00 = n2;
        m10 = d2;
        m11 = m10; // update denominators

        if next_x < 1e-9 {
            break;
        }
        x = next_x.recip();
    }

    // Check if the semi-convergent is actually closer
    let remaining_denom = (max_denom - m11) / m10;
    let n2 = m00 * remaining_denom + m01;
    let d2 = m10 * remaining_denom + m11;

    if (val - (n2 as f64 / d2 as f64)).abs() < (val - (m00 as f64 / m10 as f64)).abs() {
        (n2, d2)
    } else {
        (m00, m10)
    }
}

#[test]
fn test_limit_denominator() {
    assert_eq!(limit_denominator(1.0, 21), (1, 1));
    assert_eq!(limit_denominator(1.5, 21), (3, 2));
    assert_eq!(limit_denominator(1.33, 3), (4, 3));
}
