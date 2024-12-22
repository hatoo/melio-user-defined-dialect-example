use cmake;

fn main() {
    let dst = cmake::build("../Ch2");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=toyc-ch2");
}
