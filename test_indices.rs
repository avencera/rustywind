use rustywind_core::property_order::get_property_index;
use rustywind_core::utility_map::UTILITY_MAP;

fn main() {
    // Check touch-action index
    if let Some(idx) = get_property_index("touch-action") {
        println!("touch-action index: {}", idx);
    } else {
        println!("touch-action NOT FOUND");
    }

    // Check --tw-space-x-reverse index
    if let Some(idx) = get_property_index("--tw-space-x-reverse") {
        println!("--tw-space-x-reverse index: {}", idx);
    } else {
        println!("--tw-space-x-reverse NOT FOUND");
    }

    // Check what properties space-x-2 maps to
    if let Some(props) = UTILITY_MAP.get_properties("space-x-2") {
        println!("\nspace-x-2 maps to:");
        for prop in props {
            if let Some(idx) = get_property_index(prop) {
                println!("  {} (index: {})", prop, idx);
            } else {
                println!("  {} (NOT IN ORDER)", prop);
            }
        }

        // Find minimum index
        let min_idx = props.iter()
            .filter_map(|&p| get_property_index(p))
            .min();
        println!("  Minimum index: {:?}", min_idx);
    } else {
        println!("space-x-2 properties NOT FOUND");
    }

    // Check what properties touch-pan-down maps to
    if let Some(props) = UTILITY_MAP.get_properties("touch-pan-down") {
        println!("\ntouch-pan-down maps to:");
        for prop in props {
            if let Some(idx) = get_property_index(prop) {
                println!("  {} (index: {})", prop, idx);
            } else {
                println!("  {} (NOT IN ORDER)", prop);
            }
        }

        // Find minimum index
        let min_idx = props.iter()
            .filter_map(|&p| get_property_index(p))
            .min();
        println!("  Minimum index: {:?}", min_idx);
    } else {
        println!("touch-pan-down properties NOT FOUND");
    }
}
