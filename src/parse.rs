use std::collections::HashMap;
use std::str;
use midly::{TrackEventKind, MetaMessage, MidiMessage, Timing, TrackEvent};

#[derive(Debug)]
pub struct SongData {
    pub island: String,
    pub duration: f64,
    pub tracks: Vec<Track>,
}
#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub parts: Vec<TrackPart>,
}
#[derive(Debug)]
pub struct TrackPart {
    pub start: f64,
    pub duration: f64,
    pub sound: Option<String>,
}


#[derive(Debug)]
struct RawTrack<'a> {
    name: &'a str,
    notes: Vec<(u32, u8, u32)>,
}

pub fn parse(filename: String, world: &String) -> SongData {
    let replacements: HashMap<&str, &str> = HashMap::from([
        ("RareBox_Monster", "O_Monster"),
        ("sony_plant_Monster", "P02_Monster"),
        ("sony_air_Monster", "P01_Monster"),
        ("Accoustic Monster", "Z01_accoustic"),
        ("Banjo Monster", "Z01_banjo"),
        ("Bass Monster", "Z01_bass"),
        ("Drums Monster", "Z01_drums"),
        ("Electric1 Monster", "Z01_electricA"),
        ("Electric2 Monster", "Z01_electricB"),
        ("Mandolin Monster", "Z01_mandolin"),
        ("Vocal Monster", "Z01_vocal"),
        ("ABDE Monster", "ABDE_Monster"),
        ("BE Monster", "BE_Monster"),
        ("E Monster", "E_Monster"),
        ("BD Monster", "BD_Monster"),
        ("B Monster", "B_Monster"),
        ("ACE Monster", "ACE_Monster"),
        ("AD Monster", "AD_Monster"),
    ]);

    let file_bytes = std::fs::read(filename).unwrap();
    let file_data = midly::Smf::parse(&file_bytes).unwrap();

    let tracks: Vec<RawTrack> = file_data.tracks.iter().map(|track| {
        let name = track.iter()
            .map(|it| {
                if let TrackEventKind::Meta(MetaMessage::TrackName(m)) = it.kind {
                    str::from_utf8(m).unwrap()
                } else {
                    ""
                }
            })
            .find(|it| it.len() != 0)
            .unwrap();

        let mut start_time: u32 = 0;
        let notes: Vec<(u32, u8, u32)> = track.iter().enumerate().map(|(index, event)| {
            let delta: u32 = event.delta.as_int();
            start_time += delta;
            let sound: u8 = if let TrackEventKind::Midi { channel: _, message: MidiMessage::NoteOn { key: k, vel: _ } } = event.kind {
                k.as_int()
            } else { 255 };
            let duration: u32 = if let TrackEventKind::Midi { channel: c, message: MidiMessage::NoteOn { key: k, vel: v } } = event.kind {
                let mut index_delta: usize = 0;
                let mut end_event: TrackEvent = event.clone();
                let mut duration_time: u32 = 0;
                while end_event.kind != (TrackEventKind::Midi { channel: c, message: MidiMessage::NoteOff { key: k, vel: v } }) {
                    duration_time += end_event.delta.as_int();
                    index_delta += 1;
                    if index + index_delta >= track.len() { break; }
                    end_event = track[index + index_delta];
                }
                duration_time - delta
            } else { 0 };
            (start_time, sound, duration)
        }).filter(|it| it.1 != 255).collect();

        RawTrack { name, notes }
    }).filter(|track| track.name.ends_with("Monster") || track.name == "Bass").collect();

    let ticks_per_beat = if let Timing::Metrical(tpb) = file_data.header.timing {
        tpb.as_int()
    } else { panic!("Timing is not metrical") };
    let microseconds_per_beat = file_data.tracks.iter().map(|track| {
        track.iter().map(|event| {
            if let TrackEventKind::Meta(MetaMessage::Tempo(t)) = event.kind {
                t.as_int()
            } else { 20_000_000 }
        }).find(|it| it != &20_000_000)
    }).find(|it| it != &None).unwrap().unwrap();
    let beats_per_second: f64 = 1000000f64 / microseconds_per_beat as f64;
    let ticks_per_second: f64 = beats_per_second * ticks_per_beat as f64;

    let song_duration_ticks = file_data.tracks.iter().map(|track| {
        let mut start_time: u32 = 0;
        for event in track.iter() {
            start_time += event.delta.as_int();
        }
        start_time
    }).max().unwrap();
    let song_duration = song_duration_ticks as f64 / ticks_per_second;

    let mut result: SongData = SongData {
        island: world.clone(),
        duration: song_duration,
        tracks: vec![],
    };
    for track in tracks {
        if world == "09" && track.name == "Bass" { continue; }
        let is_dipster = regex::Regex::new(r"^Q\d\d_Monster$").unwrap().is_match(track.name);

        let mut track_data: Track = Track {
            name: if is_dipster {
                "Q_Monster"
            } else if world == "05" && track.name == "Bass" {
                "bass"
            } else if world == "18" && track.name == "Bass_Monster" {
                "Bass"
            } else if replacements.contains_key(track.name) {
                replacements.get(track.name).unwrap()
            } else {
                track.name
            }.to_string(),
            parts: vec![],
        };

        for note in track.notes {
            track_data.parts.push(TrackPart {
                start: note.0 as f64 / ticks_per_second,
                duration: note.2 as f64 / ticks_per_second,
                sound: if note.1 == 73 && track.name == "Box_Monster" {
                    None
                } else if note.1 == 102 && world == "03" && track.name == "Q05_Monster" {
                    Some("24".to_string())
                } else if world == "18" && track.name == "EW_Monster" {
                    Some(format!("{:02}", note.1 - 72))
                } else if is_dipster {
                    if world.parse::<u8>().unwrap() >= 13 {
                        Some((note.1 - 48).to_string())
                    } else {
                        Some(note.1.to_string())
                    }
                } else if note.1 == 113 && world == "03" && track.name == "AD_Monster" {
                    Some("03".to_string())
                } else {
                    Some(format!("{:02}", note.1 - 71))
                },
            })
        }
        result.tracks.push(track_data);
    }

    return result;
}