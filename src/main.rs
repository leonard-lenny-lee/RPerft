use chess_engine::*;


fn main() {
    tables::initialize_tables();
    search::perft::run_perft_bench();
}
