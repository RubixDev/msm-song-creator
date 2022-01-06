use crate::ISLAND_NAMES;
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

pub fn write(data: &SongData, world: &String, verbose: bool, data_path: String, out_path: String, speed: f32) {
    let mut out: Vec<i16> = vec![0; (data.duration * 44100.0) as usize + 10];

    for track in data.tracks.iter() {
        if verbose { println!("\x1b[90mProcessing track {}...\x1b[0m", track.name); }

        for part in track.parts.iter() {
            if part.sound == None { continue; }
            let segment_file = if track.name == "Q_Monster" {
                format!("{}/01-Q_Monster_{}.wav", data_path, part.sound.as_ref().unwrap())
            } else {
                format!("{}/{}-{}_{}.wav", data_path, data.island, track.name, part.sound.as_ref().unwrap())
            };
            let mut segment_reader = hound::WavReader::open(&segment_file).unwrap_or_else(|e| {
                eprintln!("\x1b[31mError while opening \x1b[1m{}\x1b[22m: {}\x1b[0m", segment_file, e);
                std::process::exit(10);
            });
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

    std::fs::create_dir_all(&out_path).unwrap_or_else(|e| {
        eprintln!("\x1b[31mError while creating directory \x1b[1m{}\x1b[22m: {}", out_path, e);
        std::process::exit(14);
    });

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(
        format!("{}/{}_{}.wav", out_path, world, ISLAND_NAMES[world.parse::<usize>().unwrap()].replace(' ', "-")),
        spec
    ).unwrap_or_else(|e| {
        eprintln!("\x1b[31mError while creating output file: {}\x1b[0m", e);
        std::process::exit(11);
    });

    if speed != 1.0 {
        out = resize_vec(out.clone(), (out.len() as f32 / speed).round() as usize);
    }
    for sample in out {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}
