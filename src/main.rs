fn main() {
    env_logger::init();
    chess::perft::run_perft_benchmark_suite(num_cpus::get(), 32_000_000);
}
