use std::env;

mod parse;
mod write;
mod display;

pub const ISLAND_NAMES: [&str; 22] = [
    "",
    "Plant Island",
    "Cold Island",
    "Air Island",
    "Water Island",
    "Earth Island",
    "Gold Island",
    "Ethereal Island",
    "Shugabush Island",
    "Tribal Island",
    "Wublin Island",
    "",
    "Celestial Island",
    "Fire Haven",
    "Fire Oasis",
    "Psychic Island",
    "Faerie Island",
    "Bone Island",
    "Light Island",
    "Magical Sanctum",
    "",
    "Seasonal Shanty",
];

fn main() {
    let world = env::args().collect::<Vec<String>>()[1].clone();
    let song = parse::parse(format!("data/world{}.mid", world), &world);
    write::write(&song, &world);
    display::display(&song, &world);
}
