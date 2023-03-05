fn main() {
    println!("cargo:rerun-if-changed=src/lib.h");
    println!("cargo:rerun-if-changed=src/lib.cpp");
    println!("cargo:rerun-if-changed=src/lib.rs");

    let mut cc_build = cxx_build::bridge("src/lib.rs");
    cc_build.file("src/lib.cpp");

    #[cfg(all(feature = "source", feature = "custom"))]
    compile_error!("Features 'source' and 'custom' are mutually exclusive.");

    // flags
    #[cfg(not(feature = "custom"))]
    {
        cc_build.flag("-std=c++17");
    }
    #[cfg(feature = "custom")]
    {
        if let Ok(flags) = std::env::var("CERES_RS_FLAGS") {
            for flag in flags.split(',') {
                cc_build.flag(flag);
            }
        }
    }

    // includes
    #[cfg(not(any(feature = "source", feature = "custom")))]
    {
        // Helps on Ubuntu
        #[cfg(target_os = "linux")]
        {
            cc_build.include("/usr/include/eigen3");
        }
        // Helps on x86_64 macOS with Homebrew
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            cc_build.include("/usr/local/include/eigen3");
        }
        // Helps on aarch64 macOS with Homebrew
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            cc_build.include("/opt/homebrew/include/eigen3");
        }
    }
    #[cfg(feature = "source")]
    {
        cc_build.includes(std::env::split_paths(
            &std::env::var("DEP_CERES_INCLUDE").unwrap(),
        ));
    }
    #[cfg(feature = "custom")]
    {
        if let Ok(include_dirs) = std::env::var("CERES_RS_INCLUDE_DIRS") {
            cc_build.includes(include_dirs..split(','));
        }
    }

    // defines
    #[cfg(feature = "custom")]
    {
        if let Ok(defines) = std::env::var("CERES_RS_DEFINES") {
            for define in defines..split(',') {
                if let Some((key, value)) = define.split_once('=') {
                    cc_build.define(key, value);
                }
            }
        }
    }

    cc_build.compile("ceres-solver-sys");

    #[cfg(not(any(feature = "source", feature = "custom")))]
    {
        if let Err(pkg_config_error) = pkg_config::Config::new()
            // the earliest version tested, it may work with elder versions
            .range_version("2.0.0".."3.0.0")
            .probe("ceres")
        {
            dbg!(pkg_config_error);
            println!("cargo:rustc-link-lib=ceres");
        }
    }
    #[cfg(feature = "source")]
    {
        println!("cargo:rustc-link-lib=static=ceres");
    }
    #[cfg(feature = "custom")]
    {
        if let Ok(lib_dir) = std::env::var("CERES_RS_LIB_DIR") {
            println!("cargo:rustc-link-search=native={}", lib_dir);
        }
        println!("cargo:rustc-link-lib=static=ceres");
    }
}
