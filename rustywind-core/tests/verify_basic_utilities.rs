use rustywind_core::utility_map::UtilityMap;

#[test]
fn test_basic_utilities_are_supported() {
    let map = UtilityMap::new();

    // Background utilities
    assert!(
        map.get_properties("bg-red-500").is_some(),
        "bg-red-500 should be supported"
    );
    assert!(
        map.get_properties("bg-white").is_some(),
        "bg-white should be supported"
    );
    assert!(
        map.get_properties("bg-[#fff]").is_some(),
        "bg-[#fff] arbitrary should be supported"
    );

    // Margin utilities
    assert!(
        map.get_properties("m-4").is_some(),
        "m-4 should be supported"
    );
    assert!(
        map.get_properties("mx-auto").is_some(),
        "mx-auto should be supported"
    );
    assert!(
        map.get_properties("my-8").is_some(),
        "my-8 should be supported"
    );
    assert!(
        map.get_properties("mt-2").is_some(),
        "mt-2 should be supported"
    );
    assert!(
        map.get_properties("mr-4").is_some(),
        "mr-4 should be supported"
    );
    assert!(
        map.get_properties("mb-6").is_some(),
        "mb-6 should be supported"
    );
    assert!(
        map.get_properties("ml-1").is_some(),
        "ml-1 should be supported"
    );

    // Padding utilities
    assert!(
        map.get_properties("p-4").is_some(),
        "p-4 should be supported"
    );
    assert!(
        map.get_properties("px-6").is_some(),
        "px-6 should be supported"
    );
    assert!(
        map.get_properties("py-8").is_some(),
        "py-8 should be supported"
    );
    assert!(
        map.get_properties("pt-2").is_some(),
        "pt-2 should be supported"
    );
    assert!(
        map.get_properties("pr-4").is_some(),
        "pr-4 should be supported"
    );
    assert!(
        map.get_properties("pb-6").is_some(),
        "pb-6 should be supported"
    );
    assert!(
        map.get_properties("pl-1").is_some(),
        "pl-1 should be supported"
    );

    // Text/color utilities
    assert!(
        map.get_properties("text-gray-900").is_some(),
        "text-gray-900 should be supported"
    );
    assert!(
        map.get_properties("text-white").is_some(),
        "text-white should be supported"
    );

    // Layout utilities
    assert!(
        map.get_properties("flex").is_some(),
        "flex should be supported"
    );
    assert!(
        map.get_properties("grid").is_some(),
        "grid should be supported"
    );
    assert!(
        map.get_properties("block").is_some(),
        "block should be supported"
    );
    assert!(
        map.get_properties("hidden").is_some(),
        "hidden should be supported"
    );

    // Sizing utilities
    assert!(
        map.get_properties("w-full").is_some(),
        "w-full should be supported"
    );
    assert!(
        map.get_properties("h-screen").is_some(),
        "h-screen should be supported"
    );
    assert!(
        map.get_properties("min-w-0").is_some(),
        "min-w-0 should be supported"
    );
    assert!(
        map.get_properties("max-h-96").is_some(),
        "max-h-96 should be supported"
    );

    // Border utilities
    assert!(
        map.get_properties("border").is_some(),
        "border should be supported"
    );
    assert!(
        map.get_properties("border-2").is_some(),
        "border-2 should be supported"
    );
    assert!(
        map.get_properties("border-gray-200").is_some(),
        "border-gray-200 should be supported"
    );
    assert!(
        map.get_properties("rounded-lg").is_some(),
        "rounded-lg should be supported"
    );
    assert!(
        map.get_properties("rounded-t-md").is_some(),
        "rounded-t-md should be supported"
    );

    println!("\n✓ All basic Tailwind utilities are supported!");
}
