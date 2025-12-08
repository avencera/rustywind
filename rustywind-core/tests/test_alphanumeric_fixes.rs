use rustywind_core::pattern_sorter::sort_classes;

#[test]
fn test_background_colors_alphanumeric() {
    // Test that background colors are sorted alphanumerically
    // bg-blue-900 should come before bg-green-50 (blue < green alphabetically)
    let classes = vec!["bg-blue-900", "bg-green-50"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["bg-blue-900", "bg-green-50"]);
}

#[test]
fn test_negative_rotations_absolute_values() {
    // Test that negative rotations use absolute values for sorting
    // -rotate-1 < -rotate-45 < -rotate-90 (1 < 45 < 90)
    let classes = vec!["-rotate-1", "-rotate-45", "-rotate-90"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["-rotate-1", "-rotate-45", "-rotate-90"]);
}

#[test]
fn test_negative_skew_transforms_absolute_values() {
    // Test that negative skew transforms use absolute values for sorting
    // -skew-x-1 < -skew-x-3 < -skew-x-12 (1 < 3 < 12)
    let classes = vec!["-skew-x-1", "-skew-x-3", "-skew-x-12"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["-skew-x-1", "-skew-x-3", "-skew-x-12"]);
}

#[test]
fn test_mixed_background_colors_alphanumeric() {
    // Test mixed background colors to verify alphanumeric comparison
    // Within same color: 50 < 900 (numeric comparison)
    // Between colors: blue < red (alphabetic comparison)
    let classes = vec!["bg-red-500", "bg-blue-50", "bg-blue-900", "bg-red-50"];
    let sorted = sort_classes(&classes);
    assert_eq!(
        sorted,
        vec!["bg-blue-50", "bg-blue-900", "bg-red-50", "bg-red-500"]
    );
}

#[test]
fn test_background_colors_numeric_within_color() {
    // Test that within the same color, numeric comparison works correctly
    // bg-blue-50 < bg-blue-900 (50 < 900, not lexicographic "50" > "900")
    let classes = vec!["bg-blue-900", "bg-blue-50"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["bg-blue-50", "bg-blue-900"]);
}

#[test]
fn test_negative_values_unsorted_input() {
    // Test with unsorted input to verify absolute value sorting
    let classes = vec!["-rotate-90", "-rotate-1", "-rotate-45"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["-rotate-1", "-rotate-45", "-rotate-90"]);
}

#[test]
fn test_negative_transforms_unsorted() {
    // Test with unsorted input for skew transforms
    let classes = vec!["-skew-x-12", "-skew-x-1", "-skew-x-3"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["-skew-x-1", "-skew-x-3", "-skew-x-12"]);
}
