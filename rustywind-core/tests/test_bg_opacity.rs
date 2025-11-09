use rustywind_core::property_order::get_property_index;
use rustywind_core::utility_map::UtilityMap;
use rustywind_core::pattern_sorter::PatternSorter;

#[test]
fn debug_bg_opacity_sorting() {
    let map = UtilityMap::new();

    println!("\nTesting bg-opacity-50:");
    if let Some(props) = map.get_properties("bg-opacity-50") {
        println!("  Properties: {:?}", props);
        for prop in props {
            if let Some(idx) = get_property_index(prop) {
                println!("  {} -> index {}", prop, idx);
            }
        }
    } else {
        println!("  NOT RECOGNIZED");
    }

    println!("\nTesting row-start-auto:");
    if let Some(props) = map.get_properties("row-start-auto") {
        println!("  Properties: {:?}", props);
        for prop in props {
            if let Some(idx) = get_property_index(prop) {
                println!("  {} -> index {}", prop, idx);
            }
        }
    } else {
        println!("  NOT RECOGNIZED");
    }

    println!("\nTesting sort keys:");
    let sorter = PatternSorter::new();
    for class in &["bg-opacity-50", "row-start-auto"] {
        if let Some(key) = sorter.get_sort_key(class) {
            println!("{}: variant={}, prop_idx={}, prop_count={}",
                class, key.variant_order, key.property_index, key.property_count);
        } else {
            println!("{}: NOT RECOGNIZED", class);
        }
    }
}
