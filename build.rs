//! Build script to set version information from git tags or environment

fn main() {
    // If VERSION environment variable is set (from CI), use it
    if let Ok(version) = std::env::var("VERSION") {
        println!("cargo:rustc-env=EXCELDIFF_VERSION={}", version);
        return;
    }

    // Try to get version from git describe
    match std::process::Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version.is_empty() && version != "VERGEN_IDEMPOTENT_OUTPUT" {
                // Strip 'v' prefix if present
                let version = version.strip_prefix('v').unwrap_or(&version);
                println!("cargo:rustc-env=EXCELDIFF_VERSION={}", version);
                return;
            }
        }
        _ => {}
    }

    // Fallback to "local" if not in git repo or no tags
    println!("cargo:rustc-env=EXCELDIFF_VERSION=local");
}
