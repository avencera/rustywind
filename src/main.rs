extern crate ignore;
use ignore::WalkBuilder;

fn main() {
    let walker = WalkBuilder::new("./").build();

    for file in walker {
        println!("{:?}", file.unwrap().path());
    }
}
