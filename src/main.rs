use clap::{value_parser, Arg, Command};
use rperft::*;

fn main() {
    env_logger::init();

    // Config Parser
    let fen_arg = Arg::new("fen")
        .short('f')
        .long("fen")
        .default_value(STARTING_FEN)
        .value_names([
            "PIECE PLACEMENT DATA",
            "COLOR",
            "CASTLING RIGHTS",
            "EN PASSANT TARGET SQ",
            "HALFMOVE CLOCK",
            "FULLMOVE CLOCK",
        ])
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

    let cache_arg = Arg::new("cache")
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

    let threads_args = Arg::new("threads")
        .short('t')
        .long("threads")
        .default_value("8")
        .value_name("N_THREADS")
        .value_parser(value_parser!(usize))
        .help(
            "Number of threads to use in search.\n\
            If set to 0, uses all available CPU cores.\n\
            If set to 1, multithreading is disabled",
        )
        .next_line_help(true);

    let matches = Command::new("RPerft")
        .version(VERSION)
        .author(AUTHOR)
        .about("Reasonably fast move generator")
        .arg(fen_arg)
        .arg(depth_arg)
        .arg(cache_arg)
        .arg(threads_args)
        .get_matches();

    let fen = matches.get_one::<String>("fen").expect("default arg");
    let depth = matches.get_one::<u8>("depth").expect("default arg");
    let cache_size = matches.get_one::<usize>("cache").expect("default arg");
    let num_threads = matches.get_one::<usize>("threads").expect("default arg");

    println!("{fen} {depth} {cache_size} {num_threads}");
    rperft::perft::run_perft_benchmark_suite(*num_threads, *cache_size);
}
