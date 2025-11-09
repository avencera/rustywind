use rustywind_core::utility_map::UtilityMap;

fn main() {
    let map = UtilityMap::new();

    println!("scale-100: {:?}", map.get_properties("scale-100"));
    println!("scale-x-100: {:?}", map.get_properties("scale-x-100"));
    println!("translate-x-0: {:?}", map.get_properties("translate-x-0"));
    println!("rotate-0: {:?}", map.get_properties("rotate-0"));
    println!("skew-x-0: {:?}", map.get_properties("skew-x-0"));
}
