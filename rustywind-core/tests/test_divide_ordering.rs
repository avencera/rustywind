//! Tests for divide utility ordering issues found in fuzz testing
//!
//! This test suite covers 117 failures related to divide utilities being sorted incorrectly.
//! The main issue: divide-x-reverse is being sorted in the wrong position relative to other utilities.
//!
//! From fuzz testing analysis:
//! - divide-x-reverse appears BEFORE other utilities in RustyWind, but should come AFTER
//! - divide-y-reverse has similar issues
//! - These utilities need to be ordered correctly relative to positioning, overflow, border, and other divide utilities

use rustywind_core::hybrid_sorter::HybridSorter;

#[test]
fn test_divide_reverse_vs_positioning_utilities() {
    // self-start, self-end, and other positioning utilities should come BEFORE divide-x-reverse
    let sorter = HybridSorter::new();

    let classes = vec![
        "divide-x-reverse",
        "self-start",
        "self-end",
        "self-center",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: divide-x-reverse vs positioning utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let divide_pos = sorted.iter().position(|&c| c == "divide-x-reverse").unwrap();
    let self_start_pos = sorted.iter().position(|&c| c == "self-start").unwrap();
    let self_end_pos = sorted.iter().position(|&c| c == "self-end").unwrap();
    let self_center_pos = sorted.iter().position(|&c| c == "self-center").unwrap();

    // Prettier wants positioning utilities BEFORE divide-x-reverse
    assert!(self_start_pos < divide_pos, "self-start should come before divide-x-reverse");
    assert!(self_end_pos < divide_pos, "self-end should come before divide-x-reverse");
    assert!(self_center_pos < divide_pos, "self-center should come before divide-x-reverse");
}

#[test]
fn test_divide_reverse_vs_overflow_utilities() {
    // overflow utilities should come BEFORE divide-x-reverse
    let sorter = HybridSorter::new();

    let classes = vec![
        "divide-x-reverse",
        "overflow-hidden",
        "overflow-auto",
        "overflow-x-scroll",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: divide-x-reverse vs overflow utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let divide_pos = sorted.iter().position(|&c| c == "divide-x-reverse").unwrap();
    let overflow_hidden_pos = sorted.iter().position(|&c| c == "overflow-hidden").unwrap();
    let overflow_auto_pos = sorted.iter().position(|&c| c == "overflow-auto").unwrap();
    let overflow_x_scroll_pos = sorted.iter().position(|&c| c == "overflow-x-scroll").unwrap();

    // Prettier wants overflow utilities BEFORE divide-x-reverse
    assert!(overflow_hidden_pos < divide_pos, "overflow-hidden should come before divide-x-reverse");
    assert!(overflow_auto_pos < divide_pos, "overflow-auto should come before divide-x-reverse");
    assert!(overflow_x_scroll_pos < divide_pos, "overflow-x-scroll should come before divide-x-reverse");
}

#[test]
fn test_divide_reverse_vs_other_divide_utilities() {
    // Other divide utilities (divide-double, divide-dashed, etc.) should come BEFORE divide-x-reverse
    let sorter = HybridSorter::new();

    let classes = vec![
        "divide-x-reverse",
        "divide-y-reverse",
        "divide-solid",
        "divide-dashed",
        "divide-dotted",
        "divide-double",
        "divide-none",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: divide-x-reverse vs other divide utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let divide_x_reverse_pos = sorted.iter().position(|&c| c == "divide-x-reverse").unwrap();
    let divide_y_reverse_pos = sorted.iter().position(|&c| c == "divide-y-reverse").unwrap();
    let divide_solid_pos = sorted.iter().position(|&c| c == "divide-solid").unwrap();
    let divide_dashed_pos = sorted.iter().position(|&c| c == "divide-dashed").unwrap();
    let divide_dotted_pos = sorted.iter().position(|&c| c == "divide-dotted").unwrap();
    let divide_double_pos = sorted.iter().position(|&c| c == "divide-double").unwrap();
    let divide_none_pos = sorted.iter().position(|&c| c == "divide-none").unwrap();

    // Prettier wants divide style utilities BEFORE divide-reverse utilities
    assert!(divide_solid_pos < divide_x_reverse_pos, "divide-solid should come before divide-x-reverse");
    assert!(divide_dashed_pos < divide_x_reverse_pos, "divide-dashed should come before divide-x-reverse");
    assert!(divide_dotted_pos < divide_x_reverse_pos, "divide-dotted should come before divide-x-reverse");
    assert!(divide_double_pos < divide_x_reverse_pos, "divide-double should come before divide-x-reverse");
    assert!(divide_none_pos < divide_x_reverse_pos, "divide-none should come before divide-x-reverse");

    // divide-y-reverse should also follow similar pattern
    assert!(divide_solid_pos < divide_y_reverse_pos, "divide-solid should come before divide-y-reverse");
    assert!(divide_dashed_pos < divide_y_reverse_pos, "divide-dashed should come before divide-y-reverse");
}

#[test]
fn test_divide_reverse_vs_border_utilities() {
    // border utilities should come BEFORE divide-x-reverse
    let sorter = HybridSorter::new();

    let classes = vec![
        "divide-x-reverse",
        "border",
        "border-2",
        "border-t",
        "border-solid",
        "border-gray-500",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: divide-x-reverse vs border utilities");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let divide_pos = sorted.iter().position(|&c| c == "divide-x-reverse").unwrap();
    let border_pos = sorted.iter().position(|&c| c == "border").unwrap();
    let border_2_pos = sorted.iter().position(|&c| c == "border-2").unwrap();
    let border_t_pos = sorted.iter().position(|&c| c == "border-t").unwrap();
    let border_solid_pos = sorted.iter().position(|&c| c == "border-solid").unwrap();
    let border_color_pos = sorted.iter().position(|&c| c == "border-gray-500").unwrap();

    // Prettier wants border utilities BEFORE divide-x-reverse
    assert!(border_pos < divide_pos, "border should come before divide-x-reverse");
    assert!(border_2_pos < divide_pos, "border-2 should come before divide-x-reverse");
    assert!(border_t_pos < divide_pos, "border-t should come before divide-x-reverse");
    assert!(border_solid_pos < divide_pos, "border-solid should come before divide-x-reverse");
    assert!(border_color_pos < divide_pos, "border-gray-500 should come before divide-x-reverse");
}

#[test]
fn test_divide_reverse_mixed_comprehensive() {
    // Comprehensive test with mixed utility types
    // Tests the complete ordering hierarchy
    let sorter = HybridSorter::new();

    let classes = vec![
        "divide-x-reverse",
        "divide-y-reverse",
        "self-start",
        "overflow-hidden",
        "divide-solid",
        "divide-dashed",
        "border-2",
        "border-gray-300",
        "divide-x-2",
        "divide-y-4",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: comprehensive divide-reverse ordering");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let divide_x_reverse_pos = sorted.iter().position(|&c| c == "divide-x-reverse").unwrap();
    let divide_y_reverse_pos = sorted.iter().position(|&c| c == "divide-y-reverse").unwrap();
    let self_start_pos = sorted.iter().position(|&c| c == "self-start").unwrap();
    let overflow_pos = sorted.iter().position(|&c| c == "overflow-hidden").unwrap();
    let divide_solid_pos = sorted.iter().position(|&c| c == "divide-solid").unwrap();
    let divide_dashed_pos = sorted.iter().position(|&c| c == "divide-dashed").unwrap();
    let border_2_pos = sorted.iter().position(|&c| c == "border-2").unwrap();
    let border_color_pos = sorted.iter().position(|&c| c == "border-gray-300").unwrap();
    let divide_x_2_pos = sorted.iter().position(|&c| c == "divide-x-2").unwrap();
    let _divide_y_4_pos = sorted.iter().position(|&c| c == "divide-y-4").unwrap();

    // Expected order (following Prettier):
    // 1. Positioning utilities (self-start)
    // 2. Overflow utilities (overflow-hidden)
    // 3. Border utilities (border-2, border-gray-300)
    // 4. Divide width utilities (divide-x-2, divide-y-4)
    // 5. Divide style utilities (divide-solid, divide-dashed)
    // 6. Divide reverse utilities (divide-x-reverse, divide-y-reverse)

    assert!(self_start_pos < overflow_pos, "positioning should come before overflow");
    assert!(overflow_pos < border_2_pos, "overflow should come before border");
    assert!(border_2_pos < divide_x_2_pos || border_color_pos < divide_x_2_pos, "border should come before divide width");
    assert!(divide_solid_pos < divide_x_reverse_pos, "divide style should come before divide reverse");
    assert!(divide_dashed_pos < divide_y_reverse_pos, "divide style should come before divide reverse");
}

#[test]
fn test_divide_width_vs_divide_reverse() {
    // divide width utilities (divide-x-2, divide-y-4, etc.) should come BEFORE divide-reverse
    let sorter = HybridSorter::new();

    let classes = vec![
        "divide-x-reverse",
        "divide-y-reverse",
        "divide-x",
        "divide-x-2",
        "divide-x-4",
        "divide-y",
        "divide-y-2",
        "divide-y-8",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: divide width vs divide reverse");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let divide_x_reverse_pos = sorted.iter().position(|&c| c == "divide-x-reverse").unwrap();
    let divide_y_reverse_pos = sorted.iter().position(|&c| c == "divide-y-reverse").unwrap();
    let divide_x_pos = sorted.iter().position(|&c| c == "divide-x").unwrap();
    let divide_x_2_pos = sorted.iter().position(|&c| c == "divide-x-2").unwrap();
    let divide_x_4_pos = sorted.iter().position(|&c| c == "divide-x-4").unwrap();
    let divide_y_pos = sorted.iter().position(|&c| c == "divide-y").unwrap();
    let divide_y_2_pos = sorted.iter().position(|&c| c == "divide-y-2").unwrap();
    let divide_y_8_pos = sorted.iter().position(|&c| c == "divide-y-8").unwrap();

    // All divide width utilities should come BEFORE divide-reverse utilities
    assert!(divide_x_pos < divide_x_reverse_pos, "divide-x should come before divide-x-reverse");
    assert!(divide_x_2_pos < divide_x_reverse_pos, "divide-x-2 should come before divide-x-reverse");
    assert!(divide_x_4_pos < divide_x_reverse_pos, "divide-x-4 should come before divide-x-reverse");
    assert!(divide_y_pos < divide_y_reverse_pos, "divide-y should come before divide-y-reverse");
    assert!(divide_y_2_pos < divide_y_reverse_pos, "divide-y-2 should come before divide-y-reverse");
    assert!(divide_y_8_pos < divide_y_reverse_pos, "divide-y-8 should come before divide-y-reverse");
}

#[test]
fn test_divide_color_vs_divide_reverse() {
    // divide color utilities should come in the correct position relative to divide-reverse
    let sorter = HybridSorter::new();

    let classes = vec![
        "divide-x-reverse",
        "divide-gray-300",
        "divide-blue-500",
        "divide-red-600",
        "divide-opacity-50",
    ];

    let sorted = sorter.sort_classes(&classes);

    println!("\nTest: divide color vs divide reverse");
    println!("Input:  {:?}", classes);
    println!("Output: {:?}", sorted);

    // Find positions
    let divide_x_reverse_pos = sorted.iter().position(|&c| c == "divide-x-reverse").unwrap();
    let divide_gray_pos = sorted.iter().position(|&c| c == "divide-gray-300").unwrap();
    let divide_blue_pos = sorted.iter().position(|&c| c == "divide-blue-500").unwrap();
    let divide_red_pos = sorted.iter().position(|&c| c == "divide-red-600").unwrap();
    let divide_opacity_pos = sorted.iter().position(|&c| c == "divide-opacity-50").unwrap();

    // divide color utilities should come BEFORE divide-reverse
    assert!(divide_gray_pos < divide_x_reverse_pos, "divide-gray-300 should come before divide-x-reverse");
    assert!(divide_blue_pos < divide_x_reverse_pos, "divide-blue-500 should come before divide-x-reverse");
    assert!(divide_red_pos < divide_x_reverse_pos, "divide-red-600 should come before divide-x-reverse");
    assert!(divide_opacity_pos < divide_x_reverse_pos, "divide-opacity-50 should come before divide-x-reverse");
}
