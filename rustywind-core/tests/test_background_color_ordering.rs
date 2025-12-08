use rustywind_core::hybrid_sorter::HybridSorter;

/// Test that background colors are sorted alphabetically by color name.
/// According to Prettier/Tailwind ordering, bg-* utilities should be sorted
/// alphabetically: blue → gray → green → slate (and so on).
///
/// These tests verify the fix for fuzz testing failures where background
/// colors were not being sorted in the correct relative order.

#[test]
fn test_bg_blue_vs_bg_green() {
    // bg-blue should come BEFORE bg-green (alphabetically)
    let sorter = HybridSorter::new();

    let classes = vec!["bg-green-500", "bg-blue-900"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: bg-blue-900 vs bg-green-500");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: bg-blue-900, bg-green-500
    assert_eq!(
        sorted[0], "bg-blue-900",
        "bg-blue should come before bg-green"
    );
    assert_eq!(sorted[1], "bg-green-500");
}

#[test]
fn test_bg_blue_vs_bg_green_different_shades() {
    // bg-blue should come BEFORE bg-green regardless of shade number
    let sorter = HybridSorter::new();

    let classes = vec!["bg-green-50", "bg-blue-500"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: bg-blue-500 vs bg-green-50");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: bg-blue-500, bg-green-50
    assert_eq!(
        sorted[0], "bg-blue-500",
        "bg-blue-500 should come before bg-green-50"
    );
    assert_eq!(sorted[1], "bg-green-50");
}

#[test]
fn test_bg_gray_vs_bg_slate() {
    // bg-gray should come BEFORE bg-slate (alphabetically)
    let sorter = HybridSorter::new();

    let classes = vec!["bg-slate-50", "bg-gray-500"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: bg-gray-500 vs bg-slate-50");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects: bg-gray-500, bg-slate-50
    assert_eq!(
        sorted[0], "bg-gray-500",
        "bg-gray should come before bg-slate"
    );
    assert_eq!(sorted[1], "bg-slate-50");
}

#[test]
fn test_multiple_bg_colors_alphabetical() {
    // Test all four colors together: blue → gray → green → slate
    let sorter = HybridSorter::new();

    let classes = vec!["bg-slate-200", "bg-green-400", "bg-blue-600", "bg-gray-300"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: Multiple background colors (blue, gray, green, slate)");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects alphabetical order: blue → gray → green → slate
    assert_eq!(sorted[0], "bg-blue-600", "bg-blue should be first");
    assert_eq!(sorted[1], "bg-gray-300", "bg-gray should be second");
    assert_eq!(sorted[2], "bg-green-400", "bg-green should be third");
    assert_eq!(sorted[3], "bg-slate-200", "bg-slate should be fourth");
}

#[test]
fn test_bg_colors_with_different_shades_mixed() {
    // Test multiple colors with various shade numbers (50, 500, 900)
    let sorter = HybridSorter::new();

    let classes = vec![
        "bg-slate-900",
        "bg-blue-50",
        "bg-green-500",
        "bg-gray-900",
        "bg-blue-500",
        "bg-green-50",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: Mixed background colors with different shades");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expects alphabetical by color name, then by shade within same color
    // All blues first, then grays, then greens, then slates
    assert_eq!(sorted[0], "bg-blue-50", "bg-blue-50 should be first");
    assert_eq!(sorted[1], "bg-blue-500", "bg-blue-500 should be second");
    assert_eq!(sorted[2], "bg-gray-900", "bg-gray-900 should be third");
    assert_eq!(sorted[3], "bg-green-50", "bg-green-50 should be fourth");
    assert_eq!(sorted[4], "bg-green-500", "bg-green-500 should be fifth");
    assert_eq!(sorted[5], "bg-slate-900", "bg-slate-900 should be sixth");
}

#[test]
fn test_bg_colors_mixed_with_other_utilities() {
    // Test background colors mixed with other utility classes
    let sorter = HybridSorter::new();

    let classes = vec![
        "p-4",
        "bg-green-500",
        "text-white",
        "bg-blue-600",
        "rounded-lg",
        "bg-slate-200",
        "hover:bg-gray-700",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: Background colors mixed with other utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find the indices of our background color classes in the sorted output
    let bg_blue_idx = sorted.iter().position(|c| *c == "bg-blue-600").unwrap();
    let bg_green_idx = sorted.iter().position(|c| *c == "bg-green-500").unwrap();
    let bg_slate_idx = sorted.iter().position(|c| *c == "bg-slate-200").unwrap();

    // Verify background colors maintain alphabetical order among themselves
    assert!(
        bg_blue_idx < bg_green_idx,
        "bg-blue-600 should come before bg-green-500"
    );
    assert!(
        bg_green_idx < bg_slate_idx,
        "bg-green-500 should come before bg-slate-200"
    );
}

#[test]
fn test_bg_colors_with_variants() {
    // Test background colors with variants (hover, focus, etc.)
    let sorter = HybridSorter::new();

    let classes = vec![
        "bg-slate-100",
        "hover:bg-blue-500",
        "bg-green-400",
        "focus:bg-gray-300",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: Background colors with variants (hover, focus)");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find the indices of non-variant background colors
    let bg_green_idx = sorted.iter().position(|c| *c == "bg-green-400").unwrap();
    let bg_slate_idx = sorted.iter().position(|c| *c == "bg-slate-100").unwrap();

    // Verify base background colors maintain alphabetical order
    assert!(
        bg_green_idx < bg_slate_idx,
        "bg-green-400 should come before bg-slate-100"
    );
}

#[test]
fn test_bg_colors_comprehensive_alphabet() {
    // Test a comprehensive set of background color names in alphabetical order
    let sorter = HybridSorter::new();

    let classes = vec![
        "bg-zinc-500",
        "bg-amber-500",
        "bg-cyan-500",
        "bg-blue-500",
        "bg-red-500",
        "bg-emerald-500",
        "bg-gray-500",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: Comprehensive background color alphabet");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Extract just the color names for verification
    // Expected alphabetical: amber, blue, cyan, emerald, gray, red, zinc
    let expected_order = vec![
        "bg-amber-500",
        "bg-blue-500",
        "bg-cyan-500",
        "bg-emerald-500",
        "bg-gray-500",
        "bg-red-500",
        "bg-zinc-500",
    ];

    assert_eq!(
        sorted, expected_order,
        "Background colors should be sorted alphabetically by color name"
    );
}

#[test]
fn test_bg_same_color_different_shades() {
    // Test that within the same color, shades are sorted numerically
    let sorter = HybridSorter::new();

    let classes = vec!["bg-blue-900", "bg-blue-50", "bg-blue-500", "bg-blue-100"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: Same color (blue) with different shades");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Within same color, shades should be sorted numerically: 50, 100, 500, 900
    assert_eq!(sorted[0], "bg-blue-50", "bg-blue-50 should be first");
    assert_eq!(sorted[1], "bg-blue-100", "bg-blue-100 should be second");
    assert_eq!(sorted[2], "bg-blue-500", "bg-blue-500 should be third");
    assert_eq!(sorted[3], "bg-blue-900", "bg-blue-900 should be fourth");
}
