use regex::Regex;
use serde_json::{Map, Value};
use std::{collections::HashMap, fs::File, io::Read};

pub fn get_name_map(json: &Map<String, Value>) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for (key, value) in json.iter() {
        let name = value.as_object().unwrap()["name"].as_str().unwrap();
        if out.contains_key(name) {
            out.get_mut(name).unwrap().push(key.clone());
        } else {
            out.insert(name.to_owned(), vec![key.clone()]);
        }
    }
    out
}

pub fn read_list_file(path: String) -> Vec<String> {
    let mut file = File::open(&path).unwrap_or_else(|e| {
        eprintln!(
            "\x1b[31mError while opening file \x1b[1m{}\x1b[22m: {}",
            path, e
        );
        std::process::exit(30);
    });
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap_or_else(|e| {
        eprintln!(
            "\x1b[31mError while reading file \x1b[1m{}\x1b[22m: {}",
            path, e
        );
        std::process::exit(31);
    });

    let ignore_regex = Regex::new(r"^[\t ]*(|#.*)$").unwrap();
    let mut out: Vec<String> = vec![];
    for line in content.split('\n') {
        if ignore_regex.is_match(line) {
            continue;
        }
        out.push(line.to_owned());
    }

    out
}

pub fn parse_list(list: Vec<String>, monster_names: &HashMap<String, Vec<String>>) -> Vec<Regex> {
    let mut out: Vec<Regex> = vec![];
    for (index, line) in list
        .iter()
        .map(|filter| {
            let translated = monster_names.get(filter);
            if let Some(names) = translated {
                names.clone()
            } else {
                vec![filter.clone()]
            }
        })
        .enumerate()
    {
        out.push(Regex::new(format!("^({})$", line.join("|")).as_str()).unwrap_or_else(|e| {
            eprintln!("\x1b[31mError while parsing exclude/include list at index \x1b[1m{}\x1b[22m: {}", index, e);
            std::process::exit(32);
        }));
    }
    out
}
