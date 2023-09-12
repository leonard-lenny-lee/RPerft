use clap::{value_parser, Arg, ArgAction, Command};
use rperft::*;

fn main() {
    env_logger::init();

    // Config Parser
    let fen_arg = Arg::new("fen")
        .short('f')
        .long("fen")
        .default_value(STARTING_FEN)
        .value_names(["1", "2", "3", "4", "5", "6"])
        .value_parser(clap::builder::NonEmptyStringValueParser::new())
        .help("Fen string of the position")
        .next_line_help(true);

    let depth_arg = Arg::new("depth")
        .short('d')
        .long("depth")
        .default_value("5")
        .value_name("DEPTH")
        .value_parser(value_parser!(u8))
        .help("Depth of the search");

    let cache_size_arg = Arg::new("cache")
        .short('c')
        .long("cache")
        .default_value("32000000")
        .value_name("N_BYTES")
        .value_parser(value_parser!(usize))
        .help(
            "Size of the cache in bytes. \n\
             If set to 0, caching is disabled",
        )
        .next_line_help(true);

    let singlethread_flag = Arg::new("singlethread")
        .short('s')
        .long("singlethread")
        .action(ArgAction::SetTrue)
        .help("Use only a single thread")
        .next_line_help(true);

    let matches = Command::new("RPerft")
        .version(VERSION)
        .author(AUTHOR)
        .about("Reasonably fast move generator")
        .arg(fen_arg)
        .arg(depth_arg)
        .arg(cache_size_arg)
        .arg(singlethread_flag)
        .get_matches();

    let fen = matches
        .get_many("fen")
        .expect("default args")
        .collect::<Vec<&String>>()
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>()
        .join(" ");

    let depth = matches.get_one::<u8>("depth").expect("default arg");
    let cache_size = matches.get_one::<usize>("cache").expect("default arg");
    let multithreading = !matches.get_flag("singlethread");

    // perft::perft_wrapper(fen.as_str(), *depth, *cache_size, multithreading);
    perft::run_perft_benchmark_suite(*cache_size, multithreading, false);
}
