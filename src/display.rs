use crate::{parse::SongData, ISLAND_NAMES};
use serde_json::{Map, Value};

pub fn display(data: &SongData, world: &String, monster_names: Map<String, Value>) {
    println!(
        "\n\x1b[1m{}\x1b[0m - {}bpm {:02}:{:0>5.2}m",
        ISLAND_NAMES[world.parse::<usize>().unwrap()],
        data.bpm,
        (data.duration as u64) / 60,
        data.duration - ((data.duration as u64) / 60 * 60) as f64
    );

    let beats_per_second = data.bpm as f64 / 60.0;
    let track_length = ((data.duration * beats_per_second).round() as usize / 4) as usize + 2;

    let mut tracks = data.tracks.clone();
    tracks.sort_unstable_by_key(|it| {
        let monster_name = if it.dipster == None { it.name.clone() } else { format!("Q{:02}_Monster", it.dipster.unwrap()) };
        monster_names.keys().position(|e| e == &monster_name).unwrap_or_else(|| {
            eprintln!("\x1b[31;1m{}\x1b[22m not found while sorting\x1b[0m", it.name);
            std::process::exit(1);
        })
    });
    for track in tracks.iter() {
        let monster_name = if track.dipster == None { track.name.clone() } else { format!("Q{:02}_Monster", track.dipster.unwrap()) };
        let monster_data: Map<String, Value> = monster_names.get(&monster_name).unwrap_or_else(|| {
            eprintln!("\x1b[31mNo name for \x1b[1m{}\x1b[22m found\x1b[0m", track.name);
            std::process::exit(1);
        }).as_object().unwrap().clone();
        print!("  {: >15}: ", monster_data["name"].as_str().unwrap());

        let mut track_chars: Vec<String> = vec!["".to_string(); track_length];
        track_chars = track_chars.iter().enumerate().map(|(index, _)| {
            if index % 4 == 0 { "\u{258F}".to_string() } else { " ".to_string() }
        }).collect();
        for part in track.parts.iter() {
            if part.sound == None { continue; }
            let start    = ((part.start    * beats_per_second).round() as usize / 4) as usize;
            let duration = ((part.duration * beats_per_second).round() as usize / 4) as usize;
            let char_range = start..(start + if duration == 0 { 1 } else { duration });
            for index in char_range {
                track_chars[index] = format!(
                    "\x1b[38;5;{};7m{}\x1b[0m",
                    monster_data["color"],
                    if index == start { "\u{258F}" } else { " " }
                );
            }
        }
        println!("{}", track_chars.join(""));
    }
}
