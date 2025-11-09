//! Statistical calculations

// Statistics functions - no external dependencies needed for basic stats

pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

pub fn std_dev(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    let variance = data.iter().map(|&x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    variance.sqrt()
}

pub fn min(data: &[f64]) -> f64 {
    data.iter().copied().fold(f64::INFINITY, f64::min)
}

pub fn max(data: &[f64]) -> f64 {
    data.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

pub fn range(data: &[f64]) -> f64 {
    max(data) - min(data)
}

pub fn percentile(data: &[f64], p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    if p <= 0.0 {
        return min(data);
    }
    if p >= 1.0 {
        return max(data);
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let index = p * (sorted.len() - 1) as f64;
    let lower = sorted[index.floor() as usize];
    let upper = sorted[index.ceil() as usize];

    lower + (upper - lower) * (index - index.floor())
}

pub fn p95(data: &[f64]) -> f64 {
    percentile(data, 0.95)
}

pub fn p99(data: &[f64]) -> f64 {
    percentile(data, 0.99)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(mean(&data), 3.0);
    }

    #[test]
    fn test_std_dev() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sd = std_dev(&data);
        assert!((sd - 1.5811388300841898).abs() < 1e-10);
    }

    #[test]
    fn test_percentile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 0.5), 3.0);
        assert_eq!(percentile(&data, 0.95), 4.8);
    }
}
