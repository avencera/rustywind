use rustywind_core::pattern_sorter::PatternSorter;
use rustywind_core::property_order::get_property_index;
use rustywind_core::utility_map::UtilityMap;

#[test]
fn debug_size_sorting() {
    let map = UtilityMap::new();

    println!("\nTesting size-2:");
    if let Some(props) = map.get_properties("size-2") {
        println!("  Properties: {:?}", props);
        for prop in props {
            if let Some(idx) = get_property_index(prop) {
                println!("  {} -> index {}", prop, idx);
            }
        }
    } else {
        println!("  NOT RECOGNIZED");
    }

    println!("\nTesting h-auto:");
    if let Some(props) = map.get_properties("h-auto") {
        println!("  Properties: {:?}", props);
        for prop in props {
            if let Some(idx) = get_property_index(prop) {
                println!("  {} -> index {}", prop, idx);
            }
        }
    } else {
        println!("  NOT RECOGNIZED");
    }

    println!("\nTesting w-4:");
    if let Some(props) = map.get_properties("w-4") {
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
    for class in &["size-2", "h-auto", "w-4"] {
        if let Some(key) = sorter.get_sort_key(class) {
            println!(
                "{}: variant={}, prop_idx={:?}, prop_count={}",
                class, key.variant_order, key.property_indices, key.property_count
            );
        }
    }
}
