use std::env;

fn main() {
    let current_dir = env::current_dir().expect("Connot run in current directory");

    rustywind::run(current_dir)
}
