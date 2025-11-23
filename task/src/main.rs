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

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("failed to resolve workspace root")?
        .to_path_buf();

    let mut app_path = workspace_root.join("target");
    app_path.push(profile);
    app_path.push("bundle/osx/nes-frontend.app");

    ensure_plist_foreground_keys(&app_path)?;

    app_path.push("Contents/MacOS/nes-frontend");

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

fn ensure_plist_foreground_keys(app_path: &PathBuf) -> Result<(), String> {
    let mut plist_path = app_path.clone();
    plist_path.push("Contents/Info.plist");

    let set_bool = |key: &str, value: bool| -> Result<(), String> {
        let bool_str = if value { "YES" } else { "NO" };
        let status = Command::new("plutil")
            .arg("-replace")
            .arg(key)
            .arg("-bool")
            .arg(bool_str)
            .arg(&plist_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("plutil failed setting {key} to {value}"))
        }
    };

    let set_string = |key: &str, value: &str| -> Result<(), String> {
        let status = Command::new("plutil")
            .arg("-replace")
            .arg(key)
            .arg("-string")
            .arg(value)
            .arg(&plist_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("plutil failed setting {key} to {value}"))
        }
    };

    set_bool("LSBackgroundOnly", false)?;
    set_bool("LSUIElement", false)?;
    set_bool("LSForegroundOnly", true)?;
    set_bool("LSRequiresCarbon", false)?;
    set_string("CFBundleIdentifier", "com.greg.nes-frontend")?;

    Ok(())
}
