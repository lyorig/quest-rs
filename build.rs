use std::process::Command;

fn main() {
    let hash = unsafe {
        String::from_utf8_unchecked(
            Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .expect("Could not execute git")
                .stdout,
        )
    };

    println!("cargo:rustc-env=BUILD_COMMIT_HASH={hash}",)
}
