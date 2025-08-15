use std::cmp::Ordering;
use crate::models::TestResult;

/// Safe comparison of floating point numbers, handling NaN values
pub fn safe_float_cmp(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// Extract total average milliseconds from optional statistics, with default fallback
pub fn extract_total_avg_ms<T>(
    item: &T,
    extractor: impl Fn(&T) -> Option<f64>,
    default: f64,
) -> f64 {
    extractor(item).unwrap_or(default)
}

/// Create a comparator for finding minimum response times
pub fn min_response_time_comparator<T>(
    extractor: impl Fn(&T) -> f64,
) -> impl Fn(&T, &T) -> Ordering {
    move |a, b| safe_float_cmp(extractor(a), extractor(b))
}

/// Create a comparator for finding maximum response times
pub fn max_response_time_comparator<T>(
    extractor: impl Fn(&T) -> f64,
) -> impl Fn(&T, &T) -> Ordering {
    move |a, b| safe_float_cmp(extractor(a), extractor(b))
}

/// Extract total average milliseconds from TestResult statistics
pub fn extract_test_result_avg_ms(result: &TestResult) -> f64 {
    result.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(f64::MAX)
}

/// Extract total average milliseconds from TestResult statistics with custom default
pub fn extract_test_result_avg_ms_with_default(result: &TestResult, default: f64) -> f64 {
    result.statistics.as_ref().map(|s| s.total_avg_ms).unwrap_or(default)
}

/// Create a comparator for TestResult tuples (&String, &TestResult) for minimum response time
pub fn test_result_min_comparator() -> impl Fn(&(&String, &TestResult), &(&String, &TestResult)) -> Ordering {
    |a, b| {
        let a_time = extract_test_result_avg_ms(a.1);
        let b_time = extract_test_result_avg_ms(b.1);
        safe_float_cmp(a_time, b_time)
    }
}

/// Create a comparator for TestResult tuples (&String, &TestResult) for maximum response time
pub fn test_result_max_comparator() -> impl Fn(&(&String, &TestResult), &(&String, &TestResult)) -> Ordering {
    |a, b| {
        let a_time = extract_test_result_avg_ms_with_default(a.1, 0.0);
        let b_time = extract_test_result_avg_ms_with_default(b.1, 0.0);
        safe_float_cmp(a_time, b_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_float_cmp_normal_values() {
        assert_eq!(safe_float_cmp(1.0, 2.0), Ordering::Less);
        assert_eq!(safe_float_cmp(2.0, 1.0), Ordering::Greater);
        assert_eq!(safe_float_cmp(1.0, 1.0), Ordering::Equal);
    }

    #[test]
    fn test_safe_float_cmp_nan_handling() {
        assert_eq!(safe_float_cmp(f64::NAN, 1.0), Ordering::Equal);
        assert_eq!(safe_float_cmp(1.0, f64::NAN), Ordering::Equal);
        assert_eq!(safe_float_cmp(f64::NAN, f64::NAN), Ordering::Equal);
    }

    #[test]
    fn test_safe_float_cmp_infinity() {
        assert_eq!(safe_float_cmp(f64::INFINITY, 1.0), Ordering::Greater);
        assert_eq!(safe_float_cmp(1.0, f64::INFINITY), Ordering::Less);
        assert_eq!(safe_float_cmp(f64::INFINITY, f64::INFINITY), Ordering::Equal);
        assert_eq!(safe_float_cmp(f64::NEG_INFINITY, 1.0), Ordering::Less);
    }

    #[test]
    fn test_extract_total_avg_ms() {
        let extractor = |x: &f64| Some(*x);
        assert_eq!(extract_total_avg_ms(&1.5, extractor, 0.0), 1.5);
        
        let none_extractor = |_: &f64| None;
        assert_eq!(extract_total_avg_ms(&1.5, none_extractor, 99.0), 99.0);
    }

    #[test]
    fn test_min_response_time_comparator() {
        let data = [3.0, 1.0, 2.0];
        let extractor = |x: &f64| *x;
        let comparator = min_response_time_comparator(extractor);
        
        let min_item = data.iter().min_by(|a, b| comparator(a, b));
        assert_eq!(min_item, Some(&1.0));
    }

    #[test]
    fn test_max_response_time_comparator() {
        let data = [3.0, 1.0, 2.0];
        let extractor = |x: &f64| *x;
        let comparator = max_response_time_comparator(extractor);
        
        let max_item = data.iter().max_by(|a, b| comparator(a, b));
        assert_eq!(max_item, Some(&3.0));
    }
}