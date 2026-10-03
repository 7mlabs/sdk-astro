fn main() {
    let mut build = cc::Build::new();
    // Swiss rates are used to refine roots close to zero. Keep their arithmetic
    // identical when Cargo builds the wrapper in debug or release mode.
    build.opt_level(3);
    build.define("ASTRO_MOSHIER_ONLY", None);
    build
        .include("vendor")
        .warnings(false)
        .flag_if_supported("-std=c99")
        .flag_if_supported("-fvisibility=hidden");
    for name in [
        "swedate", "swehouse", "swejpl", "swemmoon", "swemplan", "sweph", "swephlib", "swecl",
        "swehel",
    ] {
        build.file(format!("vendor/{name}.c"));
    }
    build.compile("astrology_swiss");
    println!("cargo:rerun-if-changed=vendor");
    if cfg!(target_family = "unix") {
        println!("cargo:rustc-link-lib=m");
    }
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dl");
    }
}
