use rustywind_core::class_parser::parse_class;
use rustywind_core::pattern_sorter::sort_classes;

#[test]
fn test_opacity_slash_recognition() {
    // Test which opacity classes are recognized as known vs unknown
    let test_cases = vec![
        ("text-white/60", true),      // Standard color + opacity → KNOWN
        ("bg-black/25", true),        // Standard color + opacity → KNOWN
        ("bg-red-500/50", true),      // Standard color shade + opacity → KNOWN
        ("to-stroke/0", false),       // Custom color + opacity → UNKNOWN
        ("bg-primary/20", false),     // Custom color + opacity → UNKNOWN
        ("from-stroke/0", false),     // Custom color + opacity → UNKNOWN
        ("border-gray-300/50", true), // Standard color shade + opacity → KNOWN
    ];

    for (class, should_be_known) in test_cases {
        let parsed = parse_class(class).expect("Should parse");
        let props = parsed.get_properties();
        let is_known = props.is_some();

        println!(
            "Class: {} → utility={}, value={}",
            class, parsed.utility, parsed.value
        );
        println!("  Properties: {:?}", props);
        println!(
            "  Status: {} (expected: {})",
            if is_known { "KNOWN" } else { "UNKNOWN" },
            if should_be_known { "KNOWN" } else { "UNKNOWN" }
        );

        assert_eq!(
            is_known,
            should_be_known,
            "Class {} should be {}",
            class,
            if should_be_known { "KNOWN" } else { "UNKNOWN" }
        );
    }
}

#[test]
fn test_opacity_custom_colors_sort_first() {
    // Custom colors with opacity should sort BEFORE known classes (as unknown)
    let classes = vec!["flex", "to-stroke/0"];
    let sorted = sort_classes(&classes);
    assert_eq!(
        sorted,
        vec!["to-stroke/0", "flex"],
        "Custom color to-stroke/0 should sort BEFORE known class flex"
    );

    let classes = vec!["sticky", "bg-primary/20"];
    let sorted = sort_classes(&classes);
    assert_eq!(
        sorted,
        vec!["bg-primary/20", "sticky"],
        "Custom color bg-primary/20 should sort BEFORE known class sticky"
    );
}

#[test]
fn test_opacity_standard_colors_sort_by_property() {
    // Standard colors with opacity should sort according to property order (as known)
    let classes = vec!["text-white/60", "flex"];
    let sorted = sort_classes(&classes);
    // flex (display) vs text-white/60 (color)
    // display index < color index, so flex should come first
    assert_eq!(
        sorted,
        vec!["flex", "text-white/60"],
        "flex should sort BEFORE text-white/60 (property order)"
    );

    let classes = vec!["bg-black/25", "sticky"];
    let sorted = sort_classes(&classes);
    // sticky (position) vs bg-black/25 (background-color)
    // position index < background-color index, so sticky should come first
    assert_eq!(
        sorted,
        vec!["sticky", "bg-black/25"],
        "sticky should sort BEFORE bg-black/25 (property order)"
    );
}
