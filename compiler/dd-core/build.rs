use std::process;

fn main() {
    let target = std::env::var("TARGET").unwrap();

    println!("cargo:rustc-env=BUILDRS_TARGET={target}");
    println!(
        "cargo:rustc-env=BUILDRS_RUSTC={}",
        get_rustc_version().as_deref().unwrap_or("unknown")
    );
    println!(
        "cargo:rustc-env=BUILDRS_GIT_SHA={}",
        get_git_sha().as_deref().unwrap_or("unknown")
    );
}

fn get_rustc_version() -> Option<String> {
    let rustc_executable = std::env::var("RUSTC").unwrap();

    let output = process::Command::new(rustc_executable)
        .arg("-V")
        .output()
        .ok()?;
    Some(String::from_utf8(output.stdout).unwrap())
}

fn get_git_sha() -> Option<String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let output = process::Command::new("git")
        .current_dir(&manifest_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;
    let sha = String::from_utf8(output.stdout).unwrap();

    let output = process::Command::new("git")
        .current_dir(&manifest_dir)
        .arg("status")
        .arg("--porcelain")
        .output()
        .ok()?;
    let dirty = !output.stdout.is_empty();

    Some(format!(
        "{}{}",
        sha.trim(),
        if dirty { "-dirty" } else { "" }
    ))
}
