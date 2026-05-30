//! Benchmarks for pattern-based sorting
//!
//! Run with: cargo bench

use rustywind_core::hybrid_sorter::HybridSorter;
use rustywind_core::pattern_sorter::sort_classes;

fn generate_realistic_classes(count: usize) -> Vec<String> {
    let base_classes = vec![
        "container",
        "flex",
        "grid",
        "block",
        "inline-block",
        "relative",
        "absolute",
        "fixed",
        "p-4",
        "m-4",
        "px-6",
        "py-8",
        "bg-white",
        "text-gray-900",
        "rounded-lg",
        "shadow-md",
        "border",
        "w-full",
        "h-full",
    ];

    let variant_classes = [
        "hover:bg-gray-100",
        "focus:outline-none",
        "sm:flex",
        "md:grid",
        "lg:block",
        "dark:bg-gray-900",
        "hover:shadow-lg",
        "focus:ring-2",
    ];

    let mut classes = Vec::new();
    for i in 0..count {
        if i % 3 == 0 {
            classes.push(variant_classes[i % variant_classes.len()].to_string());
        } else {
            classes.push(base_classes[i % base_classes.len()].to_string());
        }
    }
    classes
}

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    // Small class list (10 classes)
    println!("=== Benchmark: 10 classes ===");
    benchmark_sorting(10);

    println!("\n=== Benchmark: 50 classes ===");
    benchmark_sorting(50);

    println!("\n=== Benchmark: 100 classes ===");
    benchmark_sorting(100);

    println!("\n=== Benchmark: 1000 classes ===");
    benchmark_sorting(1000);

    println!("\n=== Cache effectiveness test ===");
    benchmark_cache_effectiveness();
}

fn benchmark_sorting(count: usize) {
    let classes = generate_realistic_classes(count);
    let class_refs: Vec<&str> = classes.iter().map(|s| s.as_str()).collect();

    // Benchmark pattern sorter (no cache)
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _sorted = sort_classes(&class_refs);
    }
    let pattern_duration = start.elapsed();
    println!(
        "Pattern sorter:  {:?} per sort ({} classes)",
        pattern_duration / 100,
        count
    );

    // Benchmark hybrid sorter (with cache)
    let hybrid = HybridSorter::new();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _sorted = hybrid.sort_classes(&class_refs);
    }
    let hybrid_duration = start.elapsed();
    println!(
        "Hybrid sorter:   {:?} per sort ({} classes)",
        hybrid_duration / 100,
        count
    );

    let speedup = pattern_duration.as_nanos() as f64 / hybrid_duration.as_nanos() as f64;
    println!("Speedup: {:.2}x faster with cache", speedup);
}

fn benchmark_cache_effectiveness() {
    let classes = generate_realistic_classes(100);
    let class_refs: Vec<&str> = classes.iter().map(|s| s.as_str()).collect();

    let hybrid = HybridSorter::new();

    // First pass - cold cache
    let start = std::time::Instant::now();
    let _sorted = hybrid.sort_classes(&class_refs);
    let cold_duration = start.elapsed();

    // Second pass - warm cache
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _sorted = hybrid.sort_classes(&class_refs);
    }
    let warm_duration = start.elapsed() / 100;

    println!("Cold cache (first run): {:?}", cold_duration);
    println!("Warm cache (cached):    {:?}", warm_duration);

    let speedup = cold_duration.as_nanos() as f64 / warm_duration.as_nanos() as f64;
    println!("Cache speedup: {:.2}x faster", speedup);

    let (entries, capacity) = hybrid.cache_stats();
    println!(
        "Cache entries: {} / {} ({:.1}% full)",
        entries,
        capacity,
        (entries as f64 / capacity as f64) * 100.0
    );
}
