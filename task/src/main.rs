use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bundle the frontend app and run it
    #[command(name = "run-frontend")]
    RunFrontend(RunFrontendArgs),
}

#[derive(Parser)]
struct RunFrontendArgs {
    /// Build and run the release bundle
    #[arg(long)]
    release: bool,
    /// Execute the built binary directly instead of using `open`
    #[arg(long)]
    direct: bool,
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::RunFrontend(args) => run_frontend(args.release, args.direct),
    }
}

fn run_frontend(release: bool, direct: bool) -> Result<(), String> {
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
    sync_assets_into_bundle(&workspace_root, &app_path)?;

    if direct {
        let mut binary_path = app_path.clone();
        binary_path.push("Contents/MacOS/nes-frontend");

        let status = Command::new(binary_path)
            .current_dir(&workspace_root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("app run failed with status {status}"));
        }
    } else {
        let status = Command::new("open")
            .arg(&app_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("open failed with status {status}"));
        }
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

    // A Boolean value indicating whether the app runs only in the background.
    set_bool("LSBackgroundOnly", false)?;
    // A Boolean value indicating whether the app is an agent app that runs in the
    // background and doesn’t appear in the Dock.
    set_bool("LSUIElement", false)?;
    set_string("CFBundleIdentifier", "com.greg.nes-frontend")?;

    Ok(())
}

fn sync_assets_into_bundle(
    workspace_root: &PathBuf,
    app_path: &PathBuf,
) -> Result<(), String> {
    let source = workspace_root.join("assets");
    if !source.exists() {
        return Err("assets directory not found at workspace root".into());
    }

    let resources = app_path.join("Contents/Resources");
    let dest_assets = resources.join("assets");
    if dest_assets.exists() {
        std::fs::remove_dir_all(&dest_assets).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&dest_assets).map_err(|e| e.to_string())?;
    copy_dir_recursive(&source, &dest_assets)?;

    // Ensure the icon is present (in case the bundler omitted it or we ran from direct mode).
    let icon_src = workspace_root.join("assets/macos/NesFrontend.icns");
    let icon_dst = resources.join("NesFrontend.icns");
    if icon_src.exists() {
        std::fs::copy(&icon_src, &icon_dst).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| e.to_string())?;
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
