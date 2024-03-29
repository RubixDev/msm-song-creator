use serde_json::{Map, Value};
use std::path::PathBuf;
use structopt::StructOpt;

mod display;
mod lists;
mod parse;
mod write;

pub const ISLAND_NAMES: [&str; 24] = [
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
    "Amber Island",
    "Mythical Island",
];

/// Tool to create all My Singing Monsters songs and timelines from the game files
#[derive(Debug, StructOpt)]
#[structopt(author)]
struct Msm {
    /// Island numbers or names. Required unless `--list-islands` or `--list-monsters` is used
    #[structopt(required_unless("list-islands"), required_unless("list-monsters"))]
    islands: Vec<String>,

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

    /// Set the tempo of the song. Just like in-game this will also change the pitch
    #[structopt(short = "T", long, default_value = "1.0")]
    tempo: f32,

    /// Show a list of all valid island numbers and their respective names
    #[structopt(short, long)]
    list_islands: bool,

    /// Show a list of all monsters and their names
    #[structopt(short = "L", long)]
    list_monsters: bool,

    /// List of monsters to exclude from the song. RegEx supported
    ///
    /// Use `--list-monsters` for a list of all valid monster names
    #[structopt(short = "x", long)]
    exclude: Vec<String>,

    /// Path to a file with a list of monsters to exclude from the song. Overwrites names specified by `--exclude`
    ///
    /// One monster per line, blank lines and lines starting with `#` are ignored. RegEx supported.
    /// Use `--list-monsters` for a list of all valid monster names.
    #[structopt(short = "X", long)]
    exclude_list: Option<PathBuf>,

    /// List of monsters to include in the song. RegEx supported
    ///
    /// Takes higher precedence than `--exclude` and `--exclude-list`.
    /// Use `--list-monsters` for a list of all valid monster names.
    #[structopt(short, long)]
    include: Vec<String>,

    /// Path to a file with a list of monsters to include in the song. Overwrites names specified by `--include`
    ///
    /// Takes higher precedence than `--exclude` and `--exclude-list`.
    /// One monster per line, blank lines and lines starting with `#` are ignored. RegEx supported.
    /// Use `--list-monsters` for a list of all valid monster names.
    #[structopt(short = "I", long)]
    include_list: Option<PathBuf>,

    /// How many times the song should be repeated
    #[structopt(short, long, default_value = "1")]
    repeat: u8,
}

fn main() {
    let msm = Msm::from_args();

    if msm.list_islands {
        println!("\x1b[1mList of valid islands:\x1b[0m");
        for (index, name) in ISLAND_NAMES.iter().enumerate() {
            if name == &"" {
                continue;
            }
            println!("  {: >2}: {}", index, name);
        }
        return;
    }
    let raw_monster_names: Value = serde_json::from_reader(json_comments::StripComments::new(
        &include_bytes!("res/monster_names.json")[..],
    ))
    .unwrap();
    let monster_names: Map<String, Value> = raw_monster_names.as_object().unwrap().clone();
    if msm.list_monsters {
        println!("\x1b[1mList of monsters:\x1b[0m");
        for (key, data) in monster_names.iter() {
            println!("  {: <15} -> {}", key, data["name"].as_str().unwrap());
        }
        return;
    }

    if !(0.5..=2.0).contains(&msm.tempo) {
        eprintln!(
            "\x1b[31mThe specified tempo \x1b[1m{}\x1b[22m is not between 0.5 and 2",
            msm.tempo
        );
        std::process::exit(16);
    }
    if !(1..=100).contains(&msm.repeat) {
        eprintln!(
            "\x1b[31mThe specified repeats \x1b[1m{}\x1b[22m is not between 1 and 100",
            msm.repeat
        );
        std::process::exit(17);
    }

    let data_path: String = msm
        .path
        .unwrap_or(PathBuf::from("data"))
        .to_str()
        .unwrap_or_else(|| {
            eprintln!("\x1b[31mThe specified data path is not valid UTF-8\x1b[0m");
            std::process::exit(42);
        })
        .to_owned();
    let out_path: String = msm
        .output
        .unwrap_or(PathBuf::from("."))
        .to_str()
        .unwrap_or_else(|| {
            eprintln!("\x1b[31mThe specified output path is not valid UTF-8\x1b[0m");
            std::process::exit(42);
        })
        .to_owned();

    let exclude_list_path: Option<String> = msm.exclude_list.map(|path| {
        path.to_str()
            .unwrap_or_else(|| {
                eprintln!(
                    "\x1b[31mThe specified path to the exclude list is not valid UTF-8\x1b[0m"
                );
                std::process::exit(42);
            })
            .to_owned()
    });
    let include_list_path: Option<String> = msm.include_list.map(|path| {
        path.to_str()
            .unwrap_or_else(|| {
                eprintln!(
                    "\x1b[31mThe specified path to the include list is not valid UTF-8\x1b[0m"
                );
                std::process::exit(42);
            })
            .to_owned()
    });
    let raw_exclude_list = exclude_list_path.map_or(msm.exclude, lists::read_list_file);
    let raw_include_list = include_list_path.map_or(msm.include, lists::read_list_file);
    let name_map = lists::get_name_map(&monster_names);
    let exclude_list = lists::parse_list(raw_exclude_list, &name_map);
    let include_list = lists::parse_list(raw_include_list, &name_map);

    for raw_island in msm.islands {
        let parsed_island = raw_island.parse::<u8>();
        let island: u8 = if let Ok(num) = parsed_island {
            if num >= ISLAND_NAMES.len() as u8 || ISLAND_NAMES[num as usize].is_empty() {
                None
            } else { Some(num) }
        } else {
            let pos = ISLAND_NAMES.iter().position(|it| *it == raw_island);
            if let Some(num) = pos {
                if raw_island.is_empty() {
                    None
                } else { Some(num as u8) }
            } else { None }
        }.unwrap_or_else(|| {
            eprintln!("\x1b[31mThe specified island \x1b[1m{}\x1b[22m is not valid. Use `msm --list-islands` for a list of valid islands\x1b[0m", raw_island);
            std::process::exit(15);
        });
        let world = format!("{:02}", island);

        let song = parse::parse(
            format!("{}/world{}.mid", &data_path, world),
            &world,
            &exclude_list,
            &include_list,
        );
        if !msm.no_song {
            write::write(
                &song,
                &world,
                msm.verbose,
                &data_path,
                &out_path,
                msm.tempo,
                msm.repeat,
            );
        }
        if !msm.no_timeline {
            display::display(&song, &world, &monster_names);
        }
    }
}
