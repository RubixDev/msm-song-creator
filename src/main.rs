use std::env;

mod parse;
mod write;

fn main() {
    let world = env::args().collect::<Vec<String>>()[1].clone();
    let a = parse::parse(format!("data/world{}.mid", world), &world);
    write::write(a, &world);
}
