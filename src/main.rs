use std::path::PathBuf;
use structopt::StructOpt;

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

/// Tool to create all My Singing Monsters songs and timelines from the game files
#[derive(Debug, StructOpt)]
#[structopt(author)]
struct MSM {
    /// Island number. Required unless `--list-islands` is used
    #[structopt(required_unless("list-islands"))]
    island: Option<u8>,

    /// Path to MSM data/audio/music [default: "./data/"]
    #[structopt(short, long)]
    path: Option<PathBuf>,

    /// Output folder [default: "./"]
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// Logs extra output while processing
    #[structopt(short, long)]
    verbose: bool,

    /// Suppress song timeline
    #[structopt(short = "t", long)]
    no_timeline: bool,

    /// Suppress creating song wav file
    #[structopt(short = "s", long)]
    no_song: bool,

    /// Show a list of all valid island numbers and their respective names
    #[structopt(short, long)]
    list_islands: bool,
}

fn main() {
    let msm = MSM::from_args();

    if msm.list_islands {
        println!("\x1b[1mList of valid islands:\x1b[0m");
        for (index, name) in ISLAND_NAMES.iter().enumerate() {
            if name == &"" { continue; }
            println!("  {: >2}: {}", index, name);
        }
        return;
    }

    let data_path = msm.path.unwrap_or(PathBuf::from("data")).to_str().unwrap_or_else(|| {
        eprintln!("\x1b[31mThe specified path is not valid UTF-8\x1b[0m");
        std::process::exit(42);
    }).to_owned();
    let out_path = msm.output.unwrap_or(PathBuf::from(".")).to_str().unwrap_or_else(|| {
        eprintln!("\x1b[31mThe specified path is not valid UTF-8\x1b[0m");
        std::process::exit(42);
    }).to_owned();

    let island: u8 = msm.island.unwrap();
    if [11, 20].contains(&island) || island > 21 {
        eprintln!("\x1b[31mThe specified island \x1b[1m{}\x1b[22m is not valid. Use `msm --list-islands` for a list of valid islands\x1b[0m", island);
        std::process::exit(15);
    }
    let world = format!("{:02}", island);

    let song = parse::parse(format!("{}/world{}.mid", data_path, world), &world);
    if !msm.no_song { write::write(&song, &world, msm.verbose, data_path, out_path); }
    if !msm.no_timeline { display::display(&song, &world); }
}
