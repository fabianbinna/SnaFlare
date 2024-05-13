mod snaffler_record;
mod timer;

use clap::{command, Arg, ArgAction};
use encoding_rs::WINDOWS_1252;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use snaffler_record::SnafflerRecord;
use strsim::normalized_levenshtein;
use timer::Timer;
use std::{fs::File, env, path::PathBuf, io::Read};

const SIMILARITY_THRESHOLD: f64 = 0.6;
const MAX_CHECKED_RECORDS_PER_GROUP: usize = 5;

fn load_snaffler_log(input_path: PathBuf) -> Vec<SnafflerRecord> {
    let _timer = Timer::new("load_snaffler_log");

    let mut file = File::open(&input_path)
        .expect(format!("Could not open file {}!", &input_path.display()).as_str());
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect(format!("Could not read file {}", &input_path.display()).as_str());

    // Decode the file contents. Replace WINDOWS_1252 with the appropriate encoding.
    let (cow, _encoding_used, had_errors) = WINDOWS_1252.decode(&buffer);
    if had_errors {
        panic!("Could not decode characters of file {}!", &input_path.display());
    }
    let file_content = cow.into_owned();

    // let file_content = read_to_string(input_path).expect("Failed to read file");
    file_content.par_lines()
        .filter_map(|line| {
            match line.parse::<SnafflerRecord>() {
                Ok(record) => Some(record),
                Err(_) => {
                    if line.contains("[File]") {
                        println!("Could not parse line: {}", line);
                    }
                    None
                }
            }
        })
        .collect()
}

fn group_similar_snaffler_records(file_records: Vec<SnafflerRecord>) -> Vec<Vec<SnafflerRecord>> {
    let _timer = Timer::new("group_similar_snaffler_records");

    let mut groups: Vec<Vec<SnafflerRecord>> = Vec::new();
    file_records.into_iter().for_each(|record| {
        groups.sort_by(|a,b| b.len().cmp(&a.len()));
        match groups.par_iter_mut().find_any(|group| {
            let mut rng = rand::thread_rng();
            let n = MAX_CHECKED_RECORDS_PER_GROUP.min(group.len());
            let random_entries: Vec<_> = group.as_slice().choose_multiple(&mut rng, n).collect();
            random_entries.iter().any(|r| is_similar(&r.match_context, &record.match_context))
        }) {
            Some(group) => group.push(record),
            None => groups.push(vec![record])
        }
    });
    groups
}

fn is_similar(content1: &str, content2: &str) -> bool {
    normalized_levenshtein(content1, content2) > SIMILARITY_THRESHOLD
}

fn main() {
    let matches = command!()
        .arg(Arg::new("snaffler_log").required(true))
        .arg(Arg::new("print_rule")
            .short('r')
            .help("Print matched rule")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("print_triage_level")
            .short('t')
            .help("Print triage level")
            .action(ArgAction::SetTrue))
        .get_matches();

    let snaffler_log = PathBuf::from(matches.get_one::<String>("snaffler_log").unwrap());

    if !snaffler_log.is_file() {
        eprintln!("The path specified does not point to a file: {:?}", snaffler_log);
        std::process::exit(1);
    }
    
    println!("Parsing snaffler log...");
    let file_records: Vec<SnafflerRecord> = load_snaffler_log(snaffler_log);
    println!("Loaded {} records.", file_records.len());

    println!("Grouping...");
    let groups = group_similar_snaffler_records(file_records);

    for (i, group) in groups.iter().enumerate() {
        println!("Group {}:", i + 1);
        for record in group {
            let mut log_line = format!("{}", record.filepath);

            if matches.get_flag("print_rule") {
                log_line.push_str(format!(", {}", record.rule).as_str());
            }
            
            if matches.get_flag("print_triage_level") {
                log_line.push_str(format!(", {}", record.triage_level).as_str());
            }

            println!("  {}", log_line);
        }
        println!();
    }
}