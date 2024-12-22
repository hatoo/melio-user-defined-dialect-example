use std::{
    env,
    error::Error,
    path::Path,
    process::{exit, Command},
    str,
};

const LLVM_MAJOR_VERSION: usize = 19;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let version = llvm_config("--version")?;

    if !version.starts_with(&format!("{}.", LLVM_MAJOR_VERSION)) {
        return Err(format!(
            "failed to find correct version ({}.x.x) of llvm-config (found {})",
            LLVM_MAJOR_VERSION, version
        )
        .into());
    }

    println!("cargo:rerun-if-changed=mlir/toy.cpp");
    println!("cargo:rerun-if-changed=cc");
    println!("cargo:rustc-link-search={}", llvm_config("--libdir")?);

    for name in llvm_config("--libnames")?.trim().split(' ') {
        println!("cargo:rustc-link-lib=static={}", parse_library_name(name)?);
    }

    for flag in llvm_config("--system-libs")?.trim().split(' ') {
        let flag = flag.trim_start_matches("-l");

        if flag.starts_with('/') {
            // llvm-config returns absolute paths for dynamically linked libraries.
            let path = Path::new(flag);

            println!(
                "cargo:rustc-link-search={}",
                path.parent().unwrap().display()
            );
            println!(
                "cargo:rustc-link-lib={}",
                parse_library_name(path.file_name().unwrap().to_str().unwrap())?
            );
        } else {
            println!("cargo:rustc-link-lib={}", flag);
        }
    }

    println!("cargo:rustc-link-lib=ffi");

    if let Some(name) = get_system_libcpp() {
        println!("cargo:rustc-link-lib={}", name);
    }

    std::env::set_var("CXXFLAGS", llvm_config("--cxxflags")?);
    std::env::set_var("CFLAGS", llvm_config("--cflags")?);
    println!("cargo:rustc-link-search={}", &env::var("OUT_DIR")?);

    cc::Build::new()
        .files(&["mlir/toy.cpp", "mlir/Dialect.cpp"])
        .cpp(true)
        .include("mlir")
        .include(llvm_config("--includedir")?)
        .flag(&llvm_config("--cxxflags")?)
        .flag("-Wno-unused-parameter")
        .std("c++23")
        .compile("toy");

    // println!("cargo:rustc-link-lib=static=toy");

    /*
    bindgen::builder()
        .header("mlir/toy.h")
        .clang_arg("-Imlir")
        .clang_arg(format!("-I{}", llvm_config("--includedir")?))
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap()
        .write_to_file(Path::new(&env::var("OUT_DIR")?).join("bindings.rs"))?;
    */

    Ok(())
}

fn get_system_libcpp() -> Option<&'static str> {
    if cfg!(target_env = "msvc") {
        None
    } else if cfg!(target_os = "macos") {
        Some("c++")
    } else {
        Some("stdc++")
    }
}

fn llvm_config(argument: &str) -> Result<String, Box<dyn Error>> {
    let prefix = env::var(format!("TABLEGEN_{}0_PREFIX", LLVM_MAJOR_VERSION))
        .map(|path| Path::new(&path).join("bin"))
        .unwrap_or_default();
    let call = format!(
        "{} --link-static {}",
        prefix.join("llvm-config").display(),
        argument
    );

    Ok(str::from_utf8(
        &if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &call]).output()?
        } else {
            Command::new("sh").arg("-c").arg(&call).output()?
        }
        .stdout,
    )?
    .trim()
    .to_string())
}

fn parse_library_name(name: &str) -> Result<&str, String> {
    name.strip_prefix("lib")
        .and_then(|name| name.split('.').next())
        .ok_or_else(|| format!("failed to parse library name: {}", name))
}
