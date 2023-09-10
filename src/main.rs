fn main() {
    env_logger::init();
    rperft::perft::run_perft_benchmark_suite(8, 32_000_000);
}
