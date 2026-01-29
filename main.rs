use std::fs;
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(index = 1, required = false)]
    packages: Vec<String>,

    #[arg(index = 2, last = true)]
    editor: Option<String>,

    #[arg(short,long)]
    remove: bool,

    #[arg(short,long)]
    update: bool,

    #[arg(short,long)]
    edit: bool,
}

#[derive(Deserialize)]
struct Recipe {
    steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
    program: String,
    args: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct Package {
    name: String,
    version: String,
    source: String,
    archive: String,
    dirname: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
struct PackageList {
    packages: Vec<Package>,
}

const PKG_FILE: &str = "src/packages.json";
const RECIPE_FILE: &str = "src/recipe.json";
const INSTALLED_FILE: &str = "src/installed.json";
fn remove_tar(pkg: Package){
    let status = Command::new("rm").args(["-rf",&pkg.archive]).status().unwrap_or_else(|_e| panic!("Failed to execute."));
    if !status.success(){
        println!("Sorry");
    }
}
fn load_packages() -> PackageList {
    let s = fs::read_to_string(PKG_FILE).expect("failed to read packages.json");
    serde_json::from_str(&s).expect("invalid packages.json")
}

fn load_recipe() -> Recipe {
    let s = fs::read_to_string(RECIPE_FILE).expect("failed to read recipe.json");
    serde_json::from_str(&s).expect("invalid recipe.json")
}

fn load_installed() -> PackageList {
    match fs::read_to_string(INSTALLED_FILE) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => PackageList::default(),
    }
}

fn save_installed(list: &PackageList) {
    let s = serde_json::to_string_pretty(list).expect("failed to serialize installed.json");
    fs::write(INSTALLED_FILE, s).expect("failed to write installed.json");
}

fn substitute(arg: &str, pkg: &Package) -> String {
    arg.replace("{archive}", &pkg.archive)
        .replace("{source}", &pkg.source)
}

fn run_recipe_and_record(pkg: &Package, recipe: &Recipe) {
    let mut installed = load_installed();

    for (i, step) in recipe.steps.iter().enumerate() {
        let args: Vec<String> = step.args.iter()
            .map(|a| substitute(a, pkg))
            .collect();

        println!("[{}] running: {} {}", i, step.program, args.join(" "));
        let status = Command::new(&step.program)
            .args(&args)
            .status()
            .unwrap_or_else(|e| panic!("failed to execute {}: {}", step.program, e));

        if !status.success() {
            panic!("step {} failed: {} {}", i, step.program, args.join(" "));
        }
    }

    installed.packages.push(pkg.clone());
    save_installed(&installed);
}

fn remove_pkg_and_record(pkg: &Package) {
    if let Some(dir) = &pkg.dirname {
        if !dir.is_empty() {
            if let Err(e) = fs::remove_dir_all(dir) {
                eprintln!("warning: failed to remove {}: {}", dir, e);
            }
        }
    }

    if Path::new(&pkg.archive).exists() {
        if let Err(e) = fs::remove_file(&pkg.archive) {
            eprintln!("warning: failed to remove archive {}: {}", &pkg.archive, e);
        }
    }

    let mut installed = load_installed();
    let before = installed.packages.len();
    installed.packages.retain(|p| p.name != pkg.name);
    if installed.packages.len() == before {
        eprintln!("warning: '{}' was not listed in {}", pkg.name, INSTALLED_FILE);
    }
    save_installed(&installed);
}

fn edit_recipe(editor: Option<String>) {
    let editor = editor
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| "vi".to_string());
    let status = Command::new(editor).arg(RECIPE_FILE).status()
        .expect("failed to launch editor");
    if !status.success() {
        eprintln!("editor exited with non-zero status");
    }
}

fn main() {
    let cli = Cli::parse();

    if cli.edit {
        edit_recipe(cli.editor.clone());
        return;
    }

    let packages = load_packages();
    let recipe = load_recipe();

    if cli.packages.is_empty() {
        eprintln!("no package names provided");
        std::process::exit(1);
    }

    if cli.remove {
        for name in cli.packages {
            let pkg = packages.packages.iter()
                .find(|p| p.name == name)
                .unwrap_or_else(|| panic!("package not found in {}: {}", PKG_FILE, name));
            remove_pkg_and_record(pkg);
            println!("removed {}", pkg.name);
        }
        return;
    }

    if cli.update {
        for name in cli.packages {
            let repo_pkg = packages.packages.iter()
                .find(|p| p.name == name)
                .unwrap_or_else(|| panic!("package not found in {}: {}", PKG_FILE, name));

            let installed = load_installed();
            match installed.packages.iter().find(|p| p.name == name) {
                None => {
                    println!("{} not installed — installing", name);
                    run_recipe_and_record(repo_pkg, &recipe);
                }
                Some(installed_pkg) => {
                    if installed_pkg.version != repo_pkg.version {
                        println!("updating {} from {} to {}", name, installed_pkg.version, repo_pkg.version);
                        remove_pkg_and_record(installed_pkg);
                        run_recipe_and_record(repo_pkg, &recipe);
                    } else {
                        println!("{} is up to date", name);
                    }
                }
            }
        }
        return;
    }

    for name in cli.packages {
        let pkg = packages.packages.iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("package not found in {}: {}", PKG_FILE, name));
        run_recipe_and_record(pkg, &recipe);
        remove_tar(pkg.clone());
        println!("installed {}", pkg.name);
    }
}