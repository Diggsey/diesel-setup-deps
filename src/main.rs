#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::process::Command;
use std::io::{self, stderr, Read, Write};
use std::env;
use std::fs::{self, File};
use std::path::Path;


#[derive(Deserialize, Debug)]
struct BuildPlan {
    invocations: Vec<Invocation>,
}

#[derive(Deserialize, Debug)]
struct Invocation {
    package_name: String,
    package_version: String,
    target_kind: Vec<String>,
    cwd: String
}

fn read_file(path: &Path) -> io::Result<String> {
    let mut s = String::new();
    let mut f = File::open(path)?;
    f.read_to_string(&mut s)?;
    Ok(s)
}

fn write_file(path: &Path, content: &str) -> io::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}

fn find_versions(migrations_dir: &Path) -> io::Result<Vec<String>> {
    let mut versions = Vec::new();
    for dir_entry in fs::read_dir(&migrations_dir)? {
        versions.push(dir_entry?.file_name().into_string().unwrap());
    }
    versions.sort();
    Ok(versions)
}

fn import_migrations(migrations_dir: &Path, existing_versions: &[String], invocation: Invocation) -> io::Result<()> {
    let path: &Path = invocation.cwd.as_ref();
    let marker_file = path.join("EXPORT_MIGRATIONS");
    if let Ok(relative_src_dir) = read_file(&marker_file) {
        let src_dir = path.join(relative_src_dir.trim());
        println!("Looking for migrations in `{} {}`...", invocation.package_name, invocation.package_version);

        let mut versions = find_versions(&src_dir)?;
        versions.reverse();

        let mut aggregated_up = Vec::new();
        let mut aggregated_down = Vec::new();

        for version in &versions {
            if existing_versions.contains(version) || version.starts_with("0") {
                break;
            }

            let version_path = src_dir.join(version);
            aggregated_up.push(read_file(&version_path.join("up.sql"))?);
            aggregated_down.push(read_file(&version_path.join("down.sql"))?);
        }

        aggregated_up.reverse();

        if aggregated_up.is_empty() {
            println!("> No new migrations to import");
        } else {
            let latest_version = &versions[0];
            println!("> Importing {} new migration(s) as `{}`", aggregated_up.len(), latest_version);

            let dest_dir = migrations_dir.join(latest_version);
            fs::create_dir(&dest_dir)?;

            let up_content = aggregated_up.join("\n\n");
            let down_content = aggregated_down.join("\n\n");

            write_file(&dest_dir.join("up.sql"), &up_content)?;
            write_file(&dest_dir.join("down.sql"), &down_content)?;
        }
    }
    Ok(())
}

fn main() {
    let output = Command::new("cargo")
        .args(&["build", "-Z", "unstable-options", "--build-plan"])
        .output()
        .expect("Failed to run `cargo build -Z unstable-options --build-plan`");
    
    if !output.status.success() {
        let _ = stderr().write_all(&output.stderr);
        panic!("Command `cargo build -Z unstable-options --build-plan` exited with status {}", output.status);
    }

    let build_plan: BuildPlan = serde_json::from_slice(&output.stdout[..])
        .expect("Failed to deserialize build plan");

    let migrations_dir = env::var_os("MIGRATION_DIRECTORY").map(Into::into).unwrap_or_else(|| {
        env::current_dir().unwrap().join("migrations")
    });

    let existing_versions = find_versions(&migrations_dir)
        .expect("Failed to find migrations directory");

    for invocation in build_plan.invocations {
        if invocation.target_kind.contains(&"lib".into()) {
            import_migrations(&migrations_dir, &existing_versions, invocation).unwrap();
        }
    }
    println!("Done.");
}
