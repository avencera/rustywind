//! Comprehensive benchmarks for Tailwind class sorting
//!
//! Compares:
//! - Pattern sorter (raw, no cache)
//! - Hybrid sorter (with LRU cache)
//! - Custom sorter (old HashMap-based approach)
//!
//! Run with: cargo bench
//! Run specific benchmark: cargo bench --bench comprehensive_benchmarks

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use rustywind_core::{
    app::RustyWind, hybrid_sorter::HybridSorter, pattern_sorter::sort_classes, sorter::Sorter,
};
use std::collections::HashMap;

/// Generate realistic Tailwind class lists
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
        "sticky",
        "p-4",
        "m-4",
        "px-6",
        "py-8",
        "mx-auto",
        "bg-white",
        "bg-gray-100",
        "bg-blue-500",
        "text-gray-900",
        "text-white",
        "text-sm",
        "text-lg",
        "rounded-lg",
        "rounded-md",
        "rounded-full",
        "shadow-md",
        "shadow-lg",
        "shadow-xl",
        "border",
        "border-2",
        "border-gray-200",
        "w-full",
        "w-1/2",
        "w-screen",
        "h-full",
        "h-screen",
        "h-auto",
        "flex-col",
        "flex-row",
        "items-center",
        "justify-between",
        "gap-4",
        "space-x-4",
        "space-y-2",
        "transition-colors",
        "duration-200",
        "ease-in-out",
    ];

    let variant_classes = vec![
        "hover:bg-gray-100",
        "hover:text-blue-600",
        "hover:shadow-lg",
        "focus:outline-none",
        "focus:ring-2",
        "focus:ring-blue-500",
        "sm:flex",
        "sm:hidden",
        "sm:block",
        "md:grid",
        "md:flex-row",
        "md:p-8",
        "lg:block",
        "lg:w-1/3",
        "lg:text-xl",
        "xl:grid-cols-4",
        "xl:gap-8",
        "dark:bg-gray-900",
        "dark:text-white",
        "dark:border-gray-700",
        "hover:dark:bg-gray-800",
    ];

    let mut classes = Vec::new();
    for i in 0..count {
        if i % 4 == 0 {
            classes.push(variant_classes[i % variant_classes.len()].to_string());
        } else {
            classes.push(base_classes[i % base_classes.len()].to_string());
        }
    }
    classes
}

/// Benchmark pattern sorter (no cache)
fn bench_pattern_sorter(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_sorter");

    for size in [10, 50, 100, 500].iter() {
        let classes = generate_realistic_classes(*size);
        let class_refs: Vec<&str> = classes.iter().map(|s| s.as_str()).collect();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let sorted = sort_classes(black_box(&class_refs));
                black_box(sorted);
            });
        });
    }
    group.finish();
}

/// Benchmark hybrid sorter (with cache)
fn bench_hybrid_sorter(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_sorter");

    for size in [10, 50, 100, 500].iter() {
        let classes = generate_realistic_classes(*size);
        let class_refs: Vec<&str> = classes.iter().map(|s| s.as_str()).collect();
        let sorter = HybridSorter::new();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let sorted = sorter.sort_classes(black_box(&class_refs));
                black_box(sorted);
            });
        });
    }
    group.finish();
}

/// Benchmark custom sorter (old HashMap approach) for comparison
fn bench_custom_sorter(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_sorter");

    // Create a simple custom sorter with a few classes
    let mut custom_map = HashMap::new();
    let test_classes = vec![
        "container",
        "flex",
        "grid",
        "block",
        "p-4",
        "m-4",
        "bg-white",
        "text-gray-900",
        "rounded-lg",
        "shadow-md",
    ];
    for (i, class) in test_classes.iter().enumerate() {
        custom_map.insert(class.to_string(), i);
    }

    for size in [10, 50, 100, 500].iter() {
        let classes_str = generate_realistic_classes(*size).join(" ");
        let app = RustyWind {
            sorter: Sorter::CustomSorter(custom_map.clone()),
            ..Default::default()
        };

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let sorted = app.sort_classes(black_box(&classes_str));
                black_box(sorted);
            });
        });
    }
    group.finish();
}

/// Benchmark cache cold vs warm
fn bench_cache_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effectiveness");
    let classes = generate_realistic_classes(100);
    let class_refs: Vec<&str> = classes.iter().map(|s| s.as_str()).collect();

    // Cold cache
    group.bench_function("cold_cache", |b| {
        b.iter_batched(
            || HybridSorter::new(),
            |sorter| {
                let sorted = sorter.sort_classes(black_box(&class_refs));
                black_box(sorted);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Warm cache
    let sorter = HybridSorter::new();
    // Prime the cache
    let _ = sorter.sort_classes(&class_refs);

    group.bench_function("warm_cache", |b| {
        b.iter(|| {
            let sorted = sorter.sort_classes(black_box(&class_refs));
            black_box(sorted);
        });
    });

    group.finish();
}

/// Benchmark realistic component with mixed base and variant classes
fn bench_realistic_component(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_component");

    // Realistic component class list
    let component_classes = vec![
        "flex",
        "items-center",
        "justify-between",
        "p-4",
        "px-6",
        "bg-white",
        "dark:bg-gray-900",
        "rounded-lg",
        "shadow-md",
        "border",
        "border-gray-200",
        "dark:border-gray-700",
        "hover:shadow-lg",
        "transition-all",
        "duration-200",
        "w-full",
        "max-w-4xl",
        "mx-auto",
    ];
    let class_refs: Vec<&str> = component_classes.iter().map(|s| s.as_str()).collect();

    group.throughput(Throughput::Elements(component_classes.len() as u64));

    // Pattern sorter
    group.bench_function("pattern_sorter", |b| {
        b.iter(|| {
            let sorted = sort_classes(black_box(&class_refs));
            black_box(sorted);
        });
    });

    // Hybrid sorter
    let hybrid = HybridSorter::new();
    group.bench_function("hybrid_sorter", |b| {
        b.iter(|| {
            let sorted = hybrid.sort_classes(black_box(&class_refs));
            black_box(sorted);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_pattern_sorter,
    bench_hybrid_sorter,
    bench_custom_sorter,
    bench_cache_effectiveness,
    bench_realistic_component,
);

criterion_main!(benches);
