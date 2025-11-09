use rustywind_core::utility_map::UtilityMap;

#[test]
fn check_common_utilities_coverage() {
    let map = UtilityMap::new();
    
    // Common utilities that SHOULD be recognized
    let should_work = vec![
        ("transition-colors", "transition-property"),
        ("duration-200", "transition-duration"),
        ("delay-100", "transition-delay"),
        ("p-4", "padding"),
        ("m-4", "margin"),
        ("bg-red-500", "background-color"),
        ("text-blue-600", "color"),
    ];
    
    for (utility, expected) in &should_work {
        assert!(
            map.get_properties(utility).is_some(),
            "{} should map to {} but is not recognized",
            utility,
            expected
        );
    }
    
    // Common utilities that might NOT be recognized yet
    let possibly_missing = vec![
        ("animate-spin", "animation-name"),
        ("rotate-45", "rotate"),
        ("scale-100", "scale"),
        ("translate-x-4", "translate"),
        ("overflow-hidden", "overflow"),
        ("overflow-auto", "overflow"),
        ("object-cover", "object-fit"),
        ("cursor-pointer", "cursor"),
        ("select-none", "user-select"),
        ("will-change-auto", "will-change"),
        ("appearance-none", "appearance"),
        ("resize-none", "resize"),
        ("snap-start", "scroll-snap-align"),
        ("break-words", "word-break"),
        ("outline-none", "outline-width"),
        ("blur-sm", "filter"),
        ("brightness-50", "filter"),
        ("backdrop-blur-sm", "backdrop-filter"),
    ];
    
    let mut missing = Vec::new();
    let mut found = Vec::new();
    
    for (utility, expected_prop) in &possibly_missing {
        if map.get_properties(utility).is_none() {
            missing.push(format!("{} ({})", utility, expected_prop));
        } else {
            found.push(*utility);
        }
    }
    
    println!("\n=== Utility Coverage Report ===");
    println!("Checked {} potentially missing utilities", possibly_missing.len());
    
    if !found.is_empty() {
        println!("\n✓ Already supported ({}):", found.len());
        for util in &found {
            println!("  - {}", util);
        }
    }
    
    if !missing.is_empty() {
        println!("\n✗ Not yet supported ({}):", missing.len());
        for util in &missing {
            println!("  - {}", util);
        }
    } else {
        println!("\n✓ All tested utilities are supported!");
    }
}
