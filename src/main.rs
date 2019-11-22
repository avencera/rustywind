extern crate ignore;
use ignore::WalkBuilder;

fn main() {
    let walker = WalkBuilder::new("./").build().filter_map(Result::ok);
    for file in walker {
        println!("{:?}", file.path());
    }
}
