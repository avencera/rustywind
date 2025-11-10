use rustywind_core::pattern_sorter::sort_classes;

fn main() {
    println!("Testing the fixes...\n");

    // Test 1: Background colors with alphanumeric comparison
    println!("Test 1: Background colors (alphanumeric comparison)");
    let classes = vec!["bg-blue-900", "bg-green-50"];
    let sorted = sort_classes(&classes);
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);
    println!("Expected: bg-blue-900 comes before bg-green-50 (blue < green alphabetically)");
    println!("Result: {}", if sorted == vec!["bg-blue-900", "bg-green-50"] { "✓ PASS" } else { "✗ FAIL" });
    println!();

    // Test 2: Negative rotations (with absolute values)
    println!("Test 2: Negative rotations (absolute values)");
    let classes = vec!["-rotate-1", "-rotate-45", "-rotate-90"];
    let sorted = sort_classes(&classes);
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);
    println!("Expected: -rotate-1 < -rotate-45 < -rotate-90 (1 < 45 < 90)");
    println!("Result: {}", if sorted == vec!["-rotate-1", "-rotate-45", "-rotate-90"] { "✓ PASS" } else { "✗ FAIL" });
    println!();

    // Test 3: Negative transforms (with absolute values)
    println!("Test 3: Negative skew transforms (absolute values)");
    let classes = vec!["-skew-x-1", "-skew-x-3", "-skew-x-12"];
    let sorted = sort_classes(&classes);
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);
    println!("Expected: -skew-x-1 < -skew-x-3 < -skew-x-12 (1 < 3 < 12)");
    println!("Result: {}", if sorted == vec!["-skew-x-1", "-skew-x-3", "-skew-x-12"] { "✓ PASS" } else { "✗ FAIL" });
    println!();

    // Additional test: Mix of background colors to show alphanumeric comparison
    println!("Test 4: Mixed background colors (alphanumeric comparison)");
    let classes = vec!["bg-red-500", "bg-blue-50", "bg-blue-900", "bg-red-50"];
    let sorted = sort_classes(&classes);
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);
    println!("Expected: bg-blue-50, bg-blue-900, bg-red-50, bg-red-500 (alphanumeric: blue < red, 50 < 900, 50 < 500)");
    println!("Result: {}", if sorted == vec!["bg-blue-50", "bg-blue-900", "bg-red-50", "bg-red-500"] { "✓ PASS" } else { "✗ FAIL" });
    println!();

    // Test 5: Verify that different property indexes still sort correctly
    println!("Test 5: Different utilities with negative values");
    let classes = vec!["-rotate-90", "-translate-x-1", "-rotate-1"];
    let sorted = sort_classes(&classes);
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);
    println!("Note: These may be in different property groups, so order depends on property index");
    println!("Output: {:?}", sorted);
}
