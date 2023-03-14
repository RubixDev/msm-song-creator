use crate::parse::SongData;
use crate::ISLAND_NAMES;
use lewton::inside_ogg::OggStreamReader;

fn resize_vec(vec: Vec<i16>, size: usize) -> Vec<i16> {
    let mut out: Vec<Option<i16>> = vec![None; size];
    let old_size = vec.len();
    for (index, elem) in vec.iter().enumerate() {
        out[index * size / old_size] = Some(*elem);
    }

    let mut prev_elem = out[0];
    return out
        .iter()
        .map(|elem| {
            if elem.is_none() {
                prev_elem
            } else {
                prev_elem = *elem;
                *elem
            }
            .unwrap()
        })
        .collect();
}

pub fn write(
    data: &SongData,
    world: &String,
    verbose: bool,
    data_path: &String,
    out_path: &String,
    tempo: f32,
    repeats: u8,
) {
    let mut out: Vec<i16> = vec![0; (data.duration * 44100.0) as usize + 5];

    for track in data.tracks.iter() {
        if verbose {
            println!("\x1b[90mProcessing track {}...\x1b[0m", track.name);
        }

        for part in track.parts.iter() {
            if part.sound.is_none() {
                continue;
            }

            let mut segment: Vec<i16>;

            let raw_segment_filename = if track.name == "Q_Monster" {
                format!(
                    "{}/01-Q_Monster_{}",
                    data_path,
                    part.sound.as_ref().unwrap()
                )
            } else {
                format!(
                    "{}/{}-{}_{}",
                    data_path,
                    data.island,
                    track.name,
                    part.sound.as_ref().unwrap()
                )
            };
            if std::path::PathBuf::from(format!("{}.wav", raw_segment_filename)).exists() {
                let segment_filename = format!("{}.wav", raw_segment_filename);
                let mut segment_reader =
                    hound::WavReader::open(&segment_filename).unwrap_or_else(|e| {
                        eprintln!(
                            "\x1b[31mError while opening \x1b[1m{}\x1b[22m: {}\x1b[0m",
                            segment_filename, e
                        );
                        std::process::exit(10);
                    });
                segment = segment_reader
                    .samples::<i16>()
                    .map(|it| it.unwrap())
                    .collect();
                if segment_reader.spec().sample_rate != 44100 {
                    segment = resize_vec(
                        segment,
                        (44100.0
                            * (segment_reader.duration() as f64
                                / segment_reader.spec().sample_rate as f64))
                            as usize,
                    );
                }
            } else {
                let segment_filename = format!("{}.ogg", raw_segment_filename);
                let segment_file = std::fs::File::open(&segment_filename).unwrap_or_else(|e| {
                    eprintln!(
                        "\x1b[31mError while opening \x1b[1m{}\x1b[22m: {}\x1b[0m",
                        segment_filename, e
                    );
                    std::process::exit(10);
                });
                let mut segment_reader = OggStreamReader::new(segment_file).unwrap_or_else(|e| {
                    eprintln!(
                        "\x1b[31mError while reading \x1b[1m{}\x1b[22m: {}\x1b[0m",
                        segment_filename, e
                    );
                    std::process::exit(12);
                });
                segment = vec![];
                while let Some(mut packet) = segment_reader.read_dec_packet().unwrap_or_else(|e| {
                    eprintln!(
                        "\x1b[31mError while reading \x1b[1m{}\x1b[22m: {}\x1b[0m",
                        segment_filename, e
                    );
                    std::process::exit(12);
                }) {
                    segment.append(&mut packet[0]);
                }
                if segment_reader.ident_hdr.audio_sample_rate != 44100 {
                    let segment_len = segment.len();
                    segment = resize_vec(
                        segment,
                        (44100.0
                            * (segment_len as f64
                                / segment_reader.ident_hdr.audio_sample_rate as f64))
                            as usize,
                    );
                }
            }

            for (index, sample) in segment.iter().enumerate() {
                let out_index = index + (44100f64 * part.start) as usize;
                if index as f64 > 44100.0 * part.duration + 1.0 {
                    /* println!("{}", track.name); */
                    break;
                }
                if out_index >= out.len() {
                    println!("\x1b[1;33mWarning: {} extended past song duration.\x1b[22m Cutting off...\x1b[0m", track.name);
                    break;
                }
                out[out_index] = out[out_index].saturating_add(*sample);
            }
        }
    }

    std::fs::create_dir_all(out_path).unwrap_or_else(|e| {
        eprintln!(
            "\x1b[31mError while creating directory \x1b[1m{}\x1b[22m: {}",
            out_path, e
        );
        std::process::exit(14);
    });

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(
        format!(
            "{}/{}_{}.wav",
            out_path,
            world,
            ISLAND_NAMES[world.parse::<usize>().unwrap()].replace(' ', "-")
        ),
        spec,
    )
    .unwrap_or_else(|e| {
        eprintln!("\x1b[31mError while creating output file: {}\x1b[0m", e);
        std::process::exit(11);
    });

    if tempo != 1.0 {
        out = resize_vec(out.clone(), (out.len() as f32 / tempo).round() as usize);
    }
    for _ in 0..repeats {
        for sample in out.iter() {
            writer.write_sample(*sample).unwrap();
        }
    }
    writer.finalize().unwrap();
}
