use std::arch::is_aarch64_feature_detected;

fn main() {
    // Use PEXT instruction, instead of magic hashing on x86_64 architectures
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("bmi2") {
            println!("cargo:rustc-cfg=USE_PEXT")
        }
    }
    // Determine SIMD instruction set to use for NNUE, use LLVM auto-vectorization as a fallback
    let mut simd = "AUTO";
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            simd = "NEON"
        }
    }
    #[cfg(target_arch = "x86_64")]
    {
        simd = if is_x86_feature_detected!("avx512f") {
            "AVX512"
        } else if is_x86_feature_detected!("avx2") {
            "AVX2"
        } else if is_x86_feature_detected!("sse4.1") {
            "SSE41"
        } else if is_x86_feature_detected!("ssse3") {
            "SSSE3"
        } else if is_x86_feature_detected!("sse2") {
            "SSE2"
        } else if is_x86_feature_detected!("sse") {
            "SSE"
        } else if is_x86_feature_detected!("mmx") {
            "MMX"
        }
    }
    println!("cargo:rustc-cfg=USE_{}", simd);

    if simd != "AUTO" {
        println!("cargo:rustc-cfg=VECTOR");
    }
}
