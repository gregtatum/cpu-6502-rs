use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        return Err("usage: cargo run -p task -- run-frontend [--release]".into());
    };

    match cmd.as_str() {
        "run-frontend" => {
            let release = args.next().as_deref() == Some("--release");
            run_frontend(release)
        }
        _ => Err("unknown command. try: run-frontend [--release]".into()),
    }
}

fn run_frontend(release: bool) -> Result<(), String> {
    let profile = if release { "release" } else { "debug" };

    let mut bundle = Command::new("cargo");
    bundle.arg("bundle").arg("-p").arg("nes-frontend");
    if release {
        bundle.arg("--release");
    }
    bundle.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let status = bundle.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("cargo bundle failed with status {status}"));
    }

    let mut app_path = PathBuf::from("target");
    app_path.push(profile);
    app_path.push("bundle/osx/nes-frontend.app");

    let status = Command::new("open")
        .arg(&app_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("open failed with status {status}"));
    }

    Ok(())
}
