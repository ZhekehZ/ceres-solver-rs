fn main() {
    println!("cargo:rerun-if-changed=src/lib.h");
    println!("cargo:rerun-if-changed=src/lib.cpp");
    println!("cargo:rerun-if-changed=src/lib.rs");

    let mut cc_build = cxx_build::bridge("src/lib.rs");
    cc_build.file("src/lib.cpp");

    cc_build.flag_if_supported("-std=c++17");
    cc_build.flag_if_supported("/std:c++17");


    #[cfg(feature = "source")]
    {
        cc_build.includes(std::env::split_paths(
            &std::env::var("DEP_CERES_INCLUDE").unwrap(),
        ));
        println!("cargo:rustc-link-lib=static=ceres");
    }

    #[cfg(not(feature = "source"))]
    match pkg_config::Config::new()
        // the earliest version tested, it may work with elder versions
        .range_version("2.0.0".."3.0.0")
        .probe("ceres")
    {
        Ok(ceres_lib) => {
            for (key, value) in ceres_lib.defines {
                cc_build.define(&key, value.as_deref());
            }
            cc_build.includes(ceres_lib.include_paths);
        },
        Err(pkg_config_error) => {
            dbg!(pkg_config_error);
            println!("cargo:rustc-link-lib=ceres");

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
        },
    };

    cc_build.compile("ceres-solver-sys");
}
