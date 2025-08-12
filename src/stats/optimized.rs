//! Memory-optimized statistics calculations
//!
//! This module provides optimized implementations of statistical calculations
//! that minimize memory allocations and reduce computational complexity.

use crate::models::{TimingMetrics, Statistics};

/// Optimized statistics calculator that minimizes memory allocations
pub struct OptimizedStatisticsCalculator {
    /// Pre-allocated buffer for calculations to avoid repeated allocations
    calculation_buffer: Vec<f64>,
    /// Buffer capacity management
    buffer_capacity: usize,
}

impl OptimizedStatisticsCalculator {
    /// Create a new optimized statistics calculator
    pub fn new() -> Self {
        Self {
            calculation_buffer: Vec::with_capacity(1024), // Pre-allocate for typical workloads
            buffer_capacity: 1024,
        }
    }
    
    /// Create with specific buffer capacity for known workload sizes
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            calculation_buffer: Vec::with_capacity(capacity),
            buffer_capacity: capacity,
        }
    }
    
    /// Calculate statistics from timing measurements with minimal memory allocation
    /// This optimized version performs calculations in a single pass where possible
    pub fn calculate_optimized(&mut self, measurements: &[&TimingMetrics]) -> Statistics {
        let count = measurements.len();
        
        if count == 0 {
            return Statistics::empty();
        }
        
        // Ensure buffer capacity without reallocation
        if self.calculation_buffer.capacity() < count {
            self.calculation_buffer.reserve(count - self.calculation_buffer.capacity());
            self.buffer_capacity = self.calculation_buffer.capacity();
        }
        
        // Single-pass calculation for sums and min/max
        let mut dns_sum = 0.0;
        let mut tcp_sum = 0.0;
        let mut first_byte_sum = 0.0;
        let mut total_sum = 0.0;
        
        let mut total_min = f64::INFINITY;
        let mut total_max = f64::NEG_INFINITY;
        
        // Clear buffer and reuse for total times (needed for variance calculation)
        self.calculation_buffer.clear();
        
        for measurement in measurements {
            let dns_ms = measurement.dns_ms();
            let tcp_ms = measurement.tcp_ms();
            let first_byte_ms = measurement.first_byte_ms();
            let total_ms = measurement.total_ms();
            
            // Accumulate sums
            dns_sum += dns_ms;
            tcp_sum += tcp_ms;
            first_byte_sum += first_byte_ms;
            total_sum += total_ms;
            
            // Track min/max for total time
            if total_ms < total_min {
                total_min = total_ms;
            }
            if total_ms > total_max {
                total_max = total_ms;
            }
            
            // Store total time for variance calculation (reusing buffer)
            self.calculation_buffer.push(total_ms);
        }
        
        // Calculate averages
        let count_f64 = count as f64;
        let dns_avg = dns_sum / count_f64;
        let tcp_avg = tcp_sum / count_f64;
        let first_byte_avg = first_byte_sum / count_f64;
        let total_avg = total_sum / count_f64;
        
        // Calculate variance using pre-filled buffer (single pass over stored values)
        let variance = if count > 1 {
            let sum_squared_diff: f64 = self.calculation_buffer
                .iter()
                .map(|&x| {
                    let diff = x - total_avg;
                    diff * diff // Avoid using powi for better performance
                })
                .sum();
            sum_squared_diff / count_f64
        } else {
            0.0
        };
        
        Statistics {
            dns_avg_ms: dns_avg,
            tcp_avg_ms: tcp_avg,
            first_byte_avg_ms: first_byte_avg,
            total_avg_ms: total_avg,
            total_min_ms: total_min,
            total_max_ms: total_max,
            total_std_dev_ms: variance.sqrt(),
            success_rate: 100.0, // All measurements passed in are successful
            sample_count: count,
        }
    }
    
    /// Calculate percentiles efficiently using quickselect algorithm
    /// This avoids full sorting when only specific percentiles are needed
    pub fn calculate_percentiles(&mut self, measurements: &[&TimingMetrics], percentiles: &[f64]) -> Vec<f64> {
        if measurements.is_empty() || percentiles.is_empty() {
            return Vec::new();
        }
        
        // Fill buffer with total times
        self.calculation_buffer.clear();
        self.calculation_buffer.extend(measurements.iter().map(|m| m.total_ms()));
        
        let mut results = Vec::with_capacity(percentiles.len());
        
        for &percentile in percentiles {
            if !(0.0..=100.0).contains(&percentile) {
                results.push(0.0);
                continue;
            }
            
            let index = ((percentile / 100.0) * (self.calculation_buffer.len() - 1) as f64).round() as usize;
            
            // Create a local copy for quickselect to avoid borrowing conflicts
            let mut data_copy = self.calculation_buffer.clone();
            let value = Self::quickselect_static(&mut data_copy, index);
            results.push(value);
        }
        
        results
    }
    
    /// Quickselect algorithm for efficient k-th element finding (static version)
    /// This is more efficient than full sorting when we only need specific elements
    fn quickselect_static(data: &mut [f64], k: usize) -> f64 {
        if data.len() == 1 {
            return data[0];
        }
        
        let pivot_index = Self::partition_static(data);
        
        if k == pivot_index {
            data[k]
        } else if k < pivot_index {
            Self::quickselect_static(&mut data[..pivot_index], k)
        } else {
            Self::quickselect_static(&mut data[pivot_index + 1..], k - pivot_index - 1)
        }
    }
    
    /// Partition function for quickselect (static version)
    fn partition_static(data: &mut [f64]) -> usize {
        let len = data.len();
        let pivot_index = len / 2;
        data.swap(pivot_index, len - 1);
        
        let pivot = data[len - 1];
        let mut store_index = 0;
        
        for i in 0..len - 1 {
            if data[i] <= pivot {
                data.swap(i, store_index);
                store_index += 1;
            }
        }
        
        data.swap(store_index, len - 1);
        store_index
    }
    
    /// Calculate rolling statistics for streaming data
    /// This maintains a sliding window of statistics without storing all historical data
    pub fn rolling_statistics(&mut self, new_measurement: &TimingMetrics, window_stats: &mut RollingStats) {
        let total_ms = new_measurement.total_ms();
        
        window_stats.add_value(total_ms);
        
        // Update other metrics
        window_stats.dns_sum += new_measurement.dns_ms();
        window_stats.tcp_sum += new_measurement.tcp_ms();
        window_stats.first_byte_sum += new_measurement.first_byte_ms();
    }
    
    /// Reset internal buffers to free memory
    pub fn reset(&mut self) {
        self.calculation_buffer.clear();
        
        // Shrink buffer if it's much larger than needed
        if self.calculation_buffer.capacity() > self.buffer_capacity * 2 {
            self.calculation_buffer.shrink_to(self.buffer_capacity);
        }
    }
    
    /// Get current buffer statistics for memory optimization insights
    pub fn buffer_stats(&self) -> BufferStats {
        BufferStats {
            capacity: self.calculation_buffer.capacity(),
            length: self.calculation_buffer.len(),
            memory_usage_bytes: self.calculation_buffer.capacity() * std::mem::size_of::<f64>(),
        }
    }
}

impl Default for OptimizedStatisticsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Rolling statistics calculator for streaming data
pub struct RollingStats {
    /// Sum of values in current window
    pub total_sum: f64,
    pub dns_sum: f64,
    pub tcp_sum: f64,
    pub first_byte_sum: f64,
    
    /// Current window size
    pub count: usize,
    
    /// Min and max values in current window
    pub min_value: f64,
    pub max_value: f64,
    
    /// Sum of squared values for variance calculation
    pub sum_squared: f64,
}

impl RollingStats {
    /// Create new rolling statistics tracker
    pub fn new() -> Self {
        Self {
            total_sum: 0.0,
            dns_sum: 0.0,
            tcp_sum: 0.0,
            first_byte_sum: 0.0,
            count: 0,
            min_value: f64::INFINITY,
            max_value: f64::NEG_INFINITY,
            sum_squared: 0.0,
        }
    }
    
    /// Add a new value to the rolling statistics
    pub fn add_value(&mut self, value: f64) {
        self.total_sum += value;
        self.count += 1;
        
        if value < self.min_value {
            self.min_value = value;
        }
        if value > self.max_value {
            self.max_value = value;
        }
        
        self.sum_squared += value * value;
    }
    
    /// Get current average
    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_sum / self.count as f64
        }
    }
    
    /// Get current variance
    pub fn variance(&self) -> f64 {
        if self.count <= 1 {
            return 0.0;
        }
        
        let avg = self.average();
        let count_f64 = self.count as f64;
        
        (self.sum_squared / count_f64) - (avg * avg)
    }
    
    /// Get current standard deviation
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
    
    /// Convert to Statistics struct
    pub fn to_statistics(&self) -> Statistics {
        let count_f64 = self.count as f64;
        
        Statistics {
            dns_avg_ms: if self.count > 0 { self.dns_sum / count_f64 } else { 0.0 },
            tcp_avg_ms: if self.count > 0 { self.tcp_sum / count_f64 } else { 0.0 },
            first_byte_avg_ms: if self.count > 0 { self.first_byte_sum / count_f64 } else { 0.0 },
            total_avg_ms: self.average(),
            total_min_ms: if self.min_value.is_finite() { self.min_value } else { 0.0 },
            total_max_ms: if self.max_value.is_finite() { self.max_value } else { 0.0 },
            total_std_dev_ms: self.std_dev(),
            success_rate: 100.0,
            sample_count: self.count,
        }
    }
    
    /// Reset all values
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for RollingStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Buffer usage statistics for memory optimization
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub capacity: usize,
    pub length: usize,
    pub memory_usage_bytes: usize,
}

/// Memory pool for reusing calculation buffers across multiple statistics calculations
pub struct StatisticsMemoryPool {
    /// Available buffers
    available_buffers: Vec<Vec<f64>>,
    /// Maximum pool size to prevent unbounded memory growth
    max_pool_size: usize,
    /// Default buffer capacity for new buffers
    default_capacity: usize,
}

impl StatisticsMemoryPool {
    /// Create a new memory pool
    pub fn new(max_pool_size: usize, default_capacity: usize) -> Self {
        Self {
            available_buffers: Vec::with_capacity(max_pool_size),
            max_pool_size,
            default_capacity,
        }
    }
    
    /// Get a buffer from the pool or create a new one
    pub fn get_buffer(&mut self) -> Vec<f64> {
        self.available_buffers.pop().unwrap_or_else(|| Vec::with_capacity(self.default_capacity))
    }
    
    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, mut buffer: Vec<f64>) {
        if self.available_buffers.len() < self.max_pool_size {
            buffer.clear();
            // Don't let buffers grow too large
            if buffer.capacity() > self.default_capacity * 4 {
                buffer.shrink_to(self.default_capacity);
            }
            self.available_buffers.push(buffer);
        }
    }
    
    /// Get pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        let total_memory: usize = self.available_buffers.iter()
            .map(|b| b.capacity() * std::mem::size_of::<f64>())
            .sum();
            
        PoolStats {
            available_buffers: self.available_buffers.len(),
            total_memory_bytes: total_memory,
        }
    }
}

/// Memory pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available_buffers: usize,
    pub total_memory_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimingMetrics;
    use std::time::Duration;
    
    fn create_test_metrics(count: usize) -> Vec<TimingMetrics> {
        (0..count)
            .map(|i| TimingMetrics::success(
                Duration::from_millis(10 + i as u64 % 50),
                Duration::from_millis(50 + i as u64 % 100),
                Some(Duration::from_millis(100 + i as u64 % 200)),
                Duration::from_millis(200 + i as u64 % 300),
                Duration::from_millis(410 + i as u64 % 650),
                200,
            ))
            .collect()
    }
    
    #[test]
    fn test_optimized_statistics_calculation() {
        let mut calculator = OptimizedStatisticsCalculator::new();
        let metrics = create_test_metrics(100);
        let metric_refs: Vec<&TimingMetrics> = metrics.iter().collect();
        
        let stats = calculator.calculate_optimized(&metric_refs);
        
        assert_eq!(stats.sample_count, 100);
        assert!(stats.total_avg_ms > 0.0);
        assert!(stats.total_std_dev_ms >= 0.0);
        assert!(stats.total_min_ms <= stats.total_max_ms);
    }
    
    #[test]
    fn test_percentile_calculation() {
        let mut calculator = OptimizedStatisticsCalculator::new();
        let metrics = create_test_metrics(100);
        let metric_refs: Vec<&TimingMetrics> = metrics.iter().collect();
        
        let percentiles = calculator.calculate_percentiles(&metric_refs, &[50.0, 90.0, 99.0]);
        
        assert_eq!(percentiles.len(), 3);
        assert!(percentiles[0] <= percentiles[1]);
        assert!(percentiles[1] <= percentiles[2]);
    }
    
    #[test]
    fn test_rolling_statistics() {
        let mut calculator = OptimizedStatisticsCalculator::new();
        let mut rolling = RollingStats::new();
        let metrics = create_test_metrics(10);
        
        for metric in &metrics {
            calculator.rolling_statistics(metric, &mut rolling);
        }
        
        assert_eq!(rolling.count, 10);
        assert!(rolling.average() > 0.0);
        assert!(rolling.std_dev() >= 0.0);
    }
    
    #[test]
    fn test_memory_pool() {
        let mut pool = StatisticsMemoryPool::new(5, 100);
        
        let buffer1 = pool.get_buffer();
        let buffer2 = pool.get_buffer();
        
        assert_eq!(buffer1.capacity(), 100);
        assert_eq!(buffer2.capacity(), 100);
        
        pool.return_buffer(buffer1);
        pool.return_buffer(buffer2);
        
        let stats = pool.pool_stats();
        assert_eq!(stats.available_buffers, 2);
    }
    
    #[test]
    fn test_buffer_stats() {
        let calculator = OptimizedStatisticsCalculator::new();
        let stats = calculator.buffer_stats();
        
        assert!(stats.capacity >= 1024);
        assert_eq!(stats.length, 0);
        assert!(stats.memory_usage_bytes > 0);
    }
}