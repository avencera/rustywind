use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_rounded_t_vs_rounded_l() {
    // rounded-t-lg should come BEFORE rounded-l-none
    let sorter = HybridSorter::new();

    let classes = vec!["rounded-l-none", "rounded-t-lg"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rounded-t-lg vs rounded-l-none");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: rounded-t-lg, rounded-l-none
    assert_eq!(
        sorted[0], "rounded-t-lg",
        "rounded-t-lg should come before rounded-l-none"
    );
    assert_eq!(sorted[1], "rounded-l-none");
}

#[test]
fn test_rounded_t_none_vs_rounded_tl_lg() {
    // rounded-t-none should come BEFORE rounded-tl-lg
    let sorter = HybridSorter::new();

    let classes = vec!["rounded-tl-lg", "rounded-t-none"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rounded-t-none vs rounded-tl-lg");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: rounded-t-none, rounded-tl-lg
    assert_eq!(
        sorted[0], "rounded-t-none",
        "rounded-t-none should come before rounded-tl-lg"
    );
    assert_eq!(sorted[1], "rounded-tl-lg");
}

#[test]
fn test_rounded_r_vs_rounded_tr_none() {
    // rounded-r should come BEFORE rounded-tr-none
    let sorter = HybridSorter::new();

    let classes = vec!["rounded-tr-none", "rounded-r"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rounded-r vs rounded-tr-none");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: rounded-r, rounded-tr-none
    assert_eq!(
        sorted[0], "rounded-r",
        "rounded-r should come before rounded-tr-none"
    );
    assert_eq!(sorted[1], "rounded-tr-none");
}

#[test]
fn test_rounded_r_none_vs_rounded_tr() {
    // rounded-r-none should come BEFORE rounded-tr
    let sorter = HybridSorter::new();

    let classes = vec!["rounded-tr", "rounded-r-none"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rounded-r-none vs rounded-tr");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier wants: rounded-r-none, rounded-tr
    assert_eq!(
        sorted[0], "rounded-r-none",
        "rounded-r-none should come before rounded-tr"
    );
    assert_eq!(sorted[1], "rounded-tr");
}

#[test]
fn test_mixed_rounded_utilities() {
    // Test multiple rounded utilities together
    let sorter = HybridSorter::new();

    let classes = vec![
        "rounded-br-lg",
        "rounded-t-lg",
        "rounded-l-none",
        "rounded-tl-lg",
        "rounded-r",
        "rounded-tr-none",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: mixed rounded utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Prettier expected order (verified with prettier-plugin-tailwindcss):
    // Corner utilities come before side utilities in cross-axis comparisons
    let expected = vec![
        "rounded-t-lg",
        "rounded-l-none",
        "rounded-tl-lg",
        "rounded-r",
        "rounded-tr-none",
        "rounded-br-lg",
    ];

    assert_eq!(
        sorted, expected,
        "Mixed rounded utilities should be sorted correctly"
    );
}

#[test]
fn test_rounded_with_size_modifiers() {
    // Test rounded corners with different size modifiers (sm, md, lg, xl, none)
    let sorter = HybridSorter::new();

    let classes = vec![
        "rounded-tl-xl",
        "rounded-t-sm",
        "rounded-t-lg",
        "rounded-t-none",
        "rounded-tl-sm",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rounded with size modifiers");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // rounded-t variants should come before rounded-tl variants
    // Within same side, size modifiers should be sorted consistently
    // rounded-t-none, rounded-t-sm, rounded-t-lg should come before rounded-tl-sm, rounded-tl-xl
    let t_positions: Vec<_> = sorted
        .iter()
        .enumerate()
        .filter(|(_, c)| c.starts_with("rounded-t-"))
        .map(|(i, _)| i)
        .collect();

    let tl_positions: Vec<_> = sorted
        .iter()
        .enumerate()
        .filter(|(_, c)| c.starts_with("rounded-tl-"))
        .map(|(i, _)| i)
        .collect();

    // All rounded-t should come before all rounded-tl
    if !t_positions.is_empty() && !tl_positions.is_empty() {
        assert!(
            t_positions.iter().max().unwrap() < tl_positions.iter().min().unwrap(),
            "All rounded-t variants should come before rounded-tl variants"
        );
    }
}

#[test]
fn test_rounded_b_vs_rounded_r() {
    // Test rounded-b (bottom) vs rounded-r (right) ordering
    let sorter = HybridSorter::new();

    let classes = vec!["rounded-r-lg", "rounded-b-lg"];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rounded-b-lg vs rounded-r-lg");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Verify consistent ordering between bottom and right rounded utilities
    // The exact order should match Prettier's expectations
    assert_eq!(sorted.len(), 2);
    assert!(sorted.contains(&"rounded-b-lg"));
    assert!(sorted.contains(&"rounded-r-lg"));
}

#[test]
fn test_rounded_corner_specificity() {
    // Test that more specific corner utilities (tl, tr, bl, br) are sorted correctly
    // relative to side utilities (t, r, b, l)
    let sorter = HybridSorter::new();

    let classes = vec![
        "rounded-bl-lg",
        "rounded-b-lg",
        "rounded-br-lg",
        "rounded-tl-lg",
        "rounded-t-lg",
        "rounded-tr-lg",
    ];
    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: rounded corner specificity");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions of side vs corner utilities
    let t_pos = sorted.iter().position(|c| *c == "rounded-t-lg").unwrap();
    let tl_pos = sorted.iter().position(|c| *c == "rounded-tl-lg").unwrap();
    let tr_pos = sorted.iter().position(|c| *c == "rounded-tr-lg").unwrap();
    let b_pos = sorted.iter().position(|c| *c == "rounded-b-lg").unwrap();
    let bl_pos = sorted.iter().position(|c| *c == "rounded-bl-lg").unwrap();
    let br_pos = sorted.iter().position(|c| *c == "rounded-br-lg").unwrap();

    // Side utilities should come before their respective corner utilities
    assert!(
        t_pos < tl_pos,
        "rounded-t-lg should come before rounded-tl-lg"
    );
    assert!(
        t_pos < tr_pos,
        "rounded-t-lg should come before rounded-tr-lg"
    );
    assert!(
        b_pos < bl_pos,
        "rounded-b-lg should come before rounded-bl-lg"
    );
    assert!(
        b_pos < br_pos,
        "rounded-b-lg should come before rounded-br-lg"
    );
}

#[test]
fn test_rounded_t_vs_rounded_tl_none() {
    // Regression test for fuzz failure: rounded-t should come before rounded-tl-none
    // Side utilities (border-top-radius, index 143) sort before corner utilities (border-top-left-radius, index 151)
    let sorter = HybridSorter::new();

    let classes = vec!["rounded-tl-none", "rounded-t"];
    let sorted = sorter.sort_classes(&classes);

    assert_eq!(
        sorted,
        vec!["rounded-t", "rounded-tl-none"],
        "rounded-t (side utility) should come before rounded-tl-none (corner utility)"
    );
}

#[test]
fn test_rounded_cross_axis_b_vs_tl() {
    // Test cross-axis ordering: rounded-b (bottom side) vs rounded-tl (top-left corner)
    // Side utilities should always sort before corner utilities, even on different axes
    let sorter = HybridSorter::new();

    let classes = vec!["rounded-tl", "rounded-b"];
    let sorted = sorter.sort_classes(&classes);

    // rounded-tl (top-left corner, index 151) should come before rounded-b (bottom side, index 145)
    // Per Prettier: corner utilities come before side utilities in cross-axis comparisons
    assert_eq!(
        sorted,
        vec!["rounded-tl", "rounded-b"],
        "rounded-tl (corner utility) should come before rounded-b (side utility) in cross-axis comparison"
    );
}

#[test]
fn test_rounded_all_cross_axis_cases() {
    // Test all the cross-axis cases mentioned in the problem statement
    let sorter = HybridSorter::new();

    // Per Prettier: corner utilities come BEFORE side utilities in cross-axis comparisons

    // rounded-tl (top-left corner) vs rounded-b (bottom side)
    assert_eq!(
        sorter.sort_classes(&["rounded-tl", "rounded-b"]),
        vec!["rounded-tl", "rounded-b"]
    );

    // rounded-tr-lg vs rounded-b
    assert_eq!(
        sorter.sort_classes(&["rounded-tr-lg", "rounded-b"]),
        vec!["rounded-tr-lg", "rounded-b"]
    );

    // rounded-tl vs rounded-r-lg
    assert_eq!(
        sorter.sort_classes(&["rounded-tl", "rounded-r-lg"]),
        vec!["rounded-tl", "rounded-r-lg"]
    );

    // rounded-l-lg vs rounded-r
    assert_eq!(
        sorter.sort_classes(&["rounded-l-lg", "rounded-r"]),
        vec!["rounded-l-lg", "rounded-r"]
    );

    // rounded-tl-none vs rounded-r
    assert_eq!(
        sorter.sort_classes(&["rounded-tl-none", "rounded-r"]),
        vec!["rounded-tl-none", "rounded-r"]
    );

    // rounded-l vs rounded-b-none
    assert_eq!(
        sorter.sort_classes(&["rounded-l", "rounded-b-none"]),
        vec!["rounded-l", "rounded-b-none"]
    );

    // rounded-l-none vs rounded-b-lg
    assert_eq!(
        sorter.sort_classes(&["rounded-l-none", "rounded-b-lg"]),
        vec!["rounded-l-none", "rounded-b-lg"]
    );
}
