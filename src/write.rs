use hound;
use crate::parse::SongData;

fn resize_vec(vec: Vec<i16>, size: usize) -> Vec<i16> {
    let mut out: Vec<Option<i16>> = vec![None; size];
    let old_size = vec.len();
    for (index, elem) in vec.iter().enumerate() {
        out[index * size / old_size] = Some(*elem);
    }

    let mut prev_elem = out[0];
    return out.iter().map(|elem| {
        if elem == &None {
            prev_elem
        } else {
            prev_elem = *elem;
            *elem
        }.unwrap()
    }).collect();
}

pub fn write(data: SongData, world: &String) {
    let island_names = vec![
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

    let mut out: Vec<i16> = vec![0; (data.duration * 44100.0) as usize + 10];

    for track in data.tracks.iter() {
        println!("\x1b[90mProcessing track {}...\x1b[0m", track.name);
        for part in track.parts.iter() {
            if part.sound == None { continue; }
            let segment_file = if track.name == "Q_Monster" {
                format!("data/01-Q_Monster_{}.wav", part.sound.as_ref().unwrap())
            } else {
                format!("data/{}-{}_{}.wav", data.island, track.name, part.sound.as_ref().unwrap())
            };
            let mut segment_reader = hound::WavReader::open(segment_file).unwrap();
            let mut segment: Vec<i16> = segment_reader.samples::<i16>().map(|it| it.unwrap()).collect();
            if segment_reader.spec().sample_rate != 44100 {
                segment = resize_vec(segment, (44100.0 * (segment_reader.duration() as f64 / segment_reader.spec().sample_rate as f64)) as usize);
            }
            for (index, sample) in segment.iter().enumerate() {
                let out_index = index + (44100f64 * part.start) as usize;
                if index as f64 > 44100.0 * part.duration + 1.0 { /* println!("{}", track.name); */ break; }
                if out_index >= out.len() {
                    println!("\x1b[1;33mWarning: {} extended past song duration.\x1b[22m Cutting off...\x1b[0m", track.name);
                    break;
                }
                out[out_index] = out[out_index].saturating_add(*sample);
            }
        }
    }

    std::fs::create_dir_all("songs").unwrap();

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(
        format!("songs/{}_{}.wav", world, island_names[world.parse::<usize>().unwrap()].replace(' ', "-")),
        spec
    ).unwrap();
    for sample in out {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}