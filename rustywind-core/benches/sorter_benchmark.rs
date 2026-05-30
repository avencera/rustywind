use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rustywind_core::hybrid_sorter::HybridSorter;

// Sample Tailwind classes for benchmarking
const SMALL_CLASS_SET: &[&str] = &[
    "flex",
    "items-center",
    "justify-between",
    "p-4",
    "bg-blue-500",
    "hover:bg-blue-600",
    "text-white",
    "rounded-lg",
    "shadow-md",
    "m-2",
];

const MEDIUM_CLASS_SET: &[&str] = &[
    "flex",
    "flex-col",
    "items-center",
    "justify-between",
    "p-4",
    "px-6",
    "py-8",
    "m-2",
    "mx-auto",
    "my-4",
    "bg-blue-500",
    "hover:bg-blue-600",
    "focus:bg-blue-700",
    "text-white",
    "text-lg",
    "font-bold",
    "rounded-lg",
    "rounded-t-xl",
    "shadow-md",
    "shadow-lg",
    "border",
    "border-gray-300",
    "w-full",
    "h-screen",
    "max-w-7xl",
];

const LARGE_CLASS_SET: &[&str] = &[
    "flex",
    "flex-col",
    "flex-row",
    "flex-wrap",
    "items-start",
    "items-center",
    "items-end",
    "justify-start",
    "justify-center",
    "justify-end",
    "justify-between",
    "p-1",
    "p-2",
    "p-4",
    "p-6",
    "p-8",
    "px-2",
    "px-4",
    "py-2",
    "py-4",
    "m-1",
    "m-2",
    "m-4",
    "mx-auto",
    "my-2",
    "my-4",
    "mt-8",
    "mb-4",
    "ml-2",
    "mr-2",
    "bg-white",
    "bg-gray-100",
    "bg-gray-200",
    "bg-blue-500",
    "bg-red-500",
    "hover:bg-blue-600",
    "hover:bg-red-600",
    "focus:bg-blue-700",
    "active:bg-blue-800",
    "text-black",
    "text-white",
    "text-gray-700",
    "text-sm",
    "text-base",
    "text-lg",
    "text-xl",
    "font-normal",
    "font-medium",
    "font-bold",
    "font-extrabold",
    "rounded",
    "rounded-lg",
    "rounded-xl",
    "rounded-t",
    "rounded-b",
    "rounded-l",
    "rounded-r",
    "shadow",
    "shadow-sm",
    "shadow-md",
    "shadow-lg",
    "shadow-xl",
    "border",
    "border-2",
    "border-gray-300",
    "border-blue-500",
    "w-full",
    "w-1/2",
    "w-1/3",
    "w-auto",
    "h-full",
    "h-screen",
    "h-auto",
    "max-w-sm",
    "max-w-md",
    "max-w-lg",
    "max-w-xl",
    "max-w-2xl",
    "max-w-7xl",
    "opacity-50",
    "opacity-75",
    "opacity-100",
    "transition",
    "duration-200",
    "ease-in-out",
    "cursor-pointer",
    "select-none",
    "overflow-hidden",
];

const VARIANT_HEAVY_CLASS_SET: &[&str] = &[
    "flex",
    "hover:flex",
    "focus:flex",
    "active:flex",
    "dark:flex",
    "md:flex",
    "lg:flex",
    "xl:flex",
    "2xl:flex",
    "hover:dark:flex",
    "md:hover:flex",
    "lg:focus:flex",
    "peer-hover:flex",
    "group-focus:flex",
    "peer-checked:flex",
    "p-4",
    "hover:p-4",
    "focus:p-4",
    "dark:p-4",
    "md:p-4",
    "lg:p-8",
    "bg-blue-500",
    "hover:bg-blue-600",
    "focus:bg-blue-700",
    "dark:bg-gray-800",
];

fn bench_get_sort_key(c: &mut Criterion) {
    let sorter = HybridSorter::new();

    c.bench_function("get_sort_key_single", |b| {
        b.iter(|| black_box(sorter.get_sort_key("hover:bg-blue-500")))
    });

    c.bench_function("get_sort_key_compound_variant", |b| {
        b.iter(|| black_box(sorter.get_sort_key("dark:md:hover:bg-blue-500")))
    });

    c.bench_function("get_sort_key_arbitrary", |b| {
        b.iter(|| black_box(sorter.get_sort_key("w-[120px]")))
    });
}

fn bench_sort_classes(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_classes");

    for (name, classes) in [
        ("small_10", SMALL_CLASS_SET),
        ("medium_25", MEDIUM_CLASS_SET),
        ("large_80", LARGE_CLASS_SET),
        ("variant_heavy_25", VARIANT_HEAVY_CLASS_SET),
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &classes,
            |b, &classes| {
                let sorter = HybridSorter::new();
                b.iter(|| black_box(sorter.sort_classes(classes)))
            },
        );
    }

    group.finish();
}

fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");

    // Benchmark with fresh sorter (cold cache)
    group.bench_function("cold_cache", |b| {
        b.iter(|| {
            let sorter = HybridSorter::new();
            black_box(sorter.sort_classes(LARGE_CLASS_SET))
        })
    });

    // Benchmark with warm cache
    group.bench_function("warm_cache", |b| {
        let sorter = HybridSorter::new();
        // Prime the cache
        for _ in 0..3 {
            sorter.sort_classes(LARGE_CLASS_SET);
        }
        b.iter(|| black_box(sorter.sort_classes(LARGE_CLASS_SET)))
    });

    group.finish();
}

fn bench_comparison_operations(c: &mut Criterion) {
    let sorter = HybridSorter::new();

    // Pre-compute sort keys for comparison
    let key1 = sorter.get_sort_key("p-4").unwrap();
    let key2 = sorter.get_sort_key("p-8").unwrap();
    let key3 = sorter.get_sort_key("hover:p-4").unwrap();
    let key4 = sorter.get_sort_key("bg-blue-500").unwrap();

    c.bench_function("compare_numeric", |b| b.iter(|| black_box(key1.cmp(&key2))));

    c.bench_function("compare_variant", |b| b.iter(|| black_box(key1.cmp(&key3))));

    c.bench_function("compare_property", |b| {
        b.iter(|| black_box(key1.cmp(&key4)))
    });
}

criterion_group!(
    benches,
    bench_get_sort_key,
    bench_sort_classes,
    bench_cache_performance,
    bench_comparison_operations
);
criterion_main!(benches);
