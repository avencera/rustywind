use rustywind_core::hybrid_sorter::HybridSorter;
use rustywind_core::property_order::get_property_index;

#[test]
fn test_space_y_vs_gap_y() {
    // space-y should come BEFORE gap-y according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["gap-y-4", "space-y-2"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-y-2 vs gap-y-4");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Check property indices
    // space-y maps to column-gap (per Tailwind v4's --tw-sort)
    // gap-y maps to row-gap
    let space_y_idx = get_property_index("column-gap");
    let gap_y_idx = get_property_index("row-gap");
    println!("column-gap index: {:?}", space_y_idx);
    println!("row-gap index: {:?}", gap_y_idx);

    // Prettier wants: space-y-2, gap-y-4
    // column-gap should be < row-gap for this to work
    assert_eq!(sorted[0], "space-y-2", "space-y should come before gap-y");
    assert_eq!(sorted[1], "gap-y-4");
}

#[test]
fn test_space_x_vs_gap_x() {
    // space-x should come AFTER gap-x according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["space-x-2", "gap-x-4"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-x-2 vs gap-x-4");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Check property indices
    // space-x maps to row-gap (per Tailwind v4's --tw-sort)
    // gap-x maps to column-gap
    let space_x_idx = get_property_index("row-gap");
    let gap_x_idx = get_property_index("column-gap");
    println!("row-gap index: {:?}", space_x_idx);
    println!("column-gap index: {:?}", gap_x_idx);

    // Prettier wants: gap-x-4, space-x-2
    // column-gap should be < row-gap for this to work
    assert_eq!(sorted[0], "gap-x-4", "gap-x should come before space-x");
    assert_eq!(sorted[1], "space-x-2");
}

#[test]
fn test_space_x_reverse_vs_space_y_reverse() {
    // space-y-reverse should come BEFORE space-x-reverse according to Prettier
    let sorter = HybridSorter::new();

    let classes = vec!["space-x-reverse", "space-y-reverse"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: space-x-reverse vs space-y-reverse");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: space-y-reverse, space-x-reverse
    // space-y-reverse maps to column-gap (index < row-gap)
    // space-x-reverse maps to row-gap
    assert_eq!(sorted[0], "space-y-reverse", "space-y-reverse should come before space-x-reverse");
    assert_eq!(sorted[1], "space-x-reverse");
}
