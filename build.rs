fn main() {
    // Use PEXT instruction, instead of magic hashing on x86_64 architectures
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("bmi2") {
            println!("cargo:rustc-cfg=USE_PEXT")
        }
    }
}
