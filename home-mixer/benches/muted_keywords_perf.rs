// Usage: rustc -O home-mixer/benches/muted_keywords_perf.rs && ./muted_keywords_perf

use std::time::Instant;

// Simulate the heavy operation of tokenizing/processing a string
fn simulate_process(s: &str) -> usize {
    s.len()
}

// The "Before" implementation: Clones the vector
fn implementation_before(keywords: &Vec<String>) -> usize {
    // OLD CODE: Cloned the vector
    let muted_keywords = keywords.clone();

    if muted_keywords.is_empty() {
        return 0;
    }

    let processed = muted_keywords.iter().map(|k| simulate_process(k));
    processed.sum()
}

// The "After" implementation: Uses a reference
fn implementation_after(keywords: &Vec<String>) -> usize {
    // NEW CODE: Uses reference
    let muted_keywords = keywords;

    if muted_keywords.is_empty() {
        return 0;
    }

    let processed = muted_keywords.iter().map(|k| simulate_process(k));
    processed.sum()
}

fn main() {
    // Setup data
    let keyword_count = 100;
    let iterations = 1_000_000;
    let keywords: Vec<String> = (0..keyword_count).map(|i| format!("keyword_{}", i)).collect();

    println!("Running benchmark with {} keywords over {} iterations...", keyword_count, iterations);

    // Warmup
    for _ in 0..1000 {
        implementation_before(&keywords);
        implementation_after(&keywords);
    }

    // Measure Before
    let start_before = Instant::now();
    let mut sum_before = 0;
    for _ in 0..iterations {
        sum_before += implementation_before(&keywords);
    }
    let duration_before = start_before.elapsed();

    // Measure After
    let start_after = Instant::now();
    let mut sum_after = 0;
    for _ in 0..iterations {
        sum_after += implementation_after(&keywords);
    }
    let duration_after = start_after.elapsed();

    // Verify correctness
    assert_eq!(sum_before, sum_after, "Implementations produced different results!");

    // Report
    println!("\nResults:");
    println!("Before (Clone):     {:.4} seconds", duration_before.as_secs_f64());
    println!("After (Reference):  {:.4} seconds", duration_after.as_secs_f64());

    let speedup = duration_before.as_secs_f64() / duration_after.as_secs_f64();
    println!("Speedup:            {:.2}x", speedup);
}
