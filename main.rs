use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env; // added
use std::fs;
use std::process::Command;


const RECIPE_FILE: &str = "usr/local/bin/src/binary.sh";
const INSTALLED_FILE: &str = "usr/local/bin/src/installed.json";
const SOURCE_FILE: &str = "usr/local/bin/src/source.sh";
const REMOVE_FILE: &str = "usr/local/bin/src/remove.sh";

#[derive(Parser)]
struct Cli {
    /// package names to act on
    packages: Vec<String>,

    /// choose which file to edit: "binary" or "source"
    #[arg(short, long)]
    edit: Option<String>, // changed from bool -> Option<String>

    /// remove package(s)
    #[arg(short, long)]
    remove: bool,

    /// update package(s)
    #[arg(short, long)]
    update: bool,

    #[arg(short,long)]
    search:Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Recipe {
    steps: Vec<Step>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Step {
    program: String,
    args: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Package {
    name: String,
    version: String,
    source: String,
    archive: String,
    dirname: String,
    dependencies: Vec<String>,
    recipe: Option<Recipe>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
struct PackageList {
    packages: Vec<Package>,
}

/* ---------- IO helpers ---------- */



fn load_packages(url: &str) -> PackageList {
    let pkg_url = url;
    let response = reqwest::blocking::get(pkg_url)
        .unwrap_or_else(|e| panic!("failed to fetch package index from {}: {}", pkg_url, e));

    if !response.status().is_success() {
        panic!(
            "package index server returned {} for {}",
            response.status(),
            pkg_url
        );
    }

    let body = response.text().unwrap_or_else(|e| {
        panic!(
            "failed to read package index response from {}: {}",
            pkg_url, e
        )
    });

    serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("invalid package index from {}: {}", pkg_url, e))
}

fn load_installed() -> PackageList {
    match fs::read_to_string(INSTALLED_FILE) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => PackageList::default(),
    }
}

fn load_global_recipe() -> Option<Recipe> {
    match fs::read_to_string(RECIPE_FILE) {
        Ok(s) => serde_json::from_str(&s).ok(),
        Err(_) => None,
    }
}

/* ---------- small helpers ---------- */

fn substitute(arg: &str, pkg: &Package) -> String {
    arg.replace("{archive}", &pkg.archive)
        .replace("{source}", &pkg.source)
        .replace("{dirname}", &pkg.dirname)
        .replace("{version}", &pkg.version)
        .replace("{name}", &pkg.name)
}

fn run_step(step: &Step, pkg: &Package) {
    let args: Vec<String> = step.args.iter().map(|a| substitute(a, pkg)).collect();
    println!("run: {} {}", step.program, args.join(" "));
    let status = Command::new(&step.program)
        .args(&args)
        .status()
        .unwrap_or_else(|e| panic!("failed to run {}: {}", step.program, e));
    if !status.success() {
        panic!("command failed: {} {}", step.program, args.join(" "));
    }
}

/* ---------- recipe / install ---------- */

fn run_recipe(pkg: &Package, global_recipe: Option<&Recipe>) {
    let recipe = pkg
        .recipe
        .as_ref()
        .or(global_recipe)
        .unwrap_or_else(|| panic!("no recipe for package {} and no global recipe", pkg.name));
    for step in &recipe.steps {
        run_step(step, pkg);
    }
    // do not touch installed.json here; the invoked script/recipe should record installation
}

/* ---------- removal helpers ---------- */

fn run_remove_script(dirname: &str) {
    let status = Command::new("sh")
        .args(["./src/remove.sh", dirname])
        .status()
        .unwrap_or_else(|e| panic!("failed to execute remove.sh: {}", e));
    if !status.success() {
        eprintln!(
            "warning: remove.sh exited with non-zero status for '{}'",
            dirname
        );
    }
}

/* ---------- find helpers ---------- */

fn find_in_repo<'a>(packages: &'a PackageList, name: &str) -> Option<&'a Package> {
    packages.packages.iter().find(|p| p.name == name)
}

fn find_installed_by_dirname<'a>(installed: &'a PackageList, dirname: &str) -> Option<&'a Package> {
    installed.packages.iter().find(|p| p.dirname == dirname)
}

/* ---------- recursive operations ---------- */

fn install_recursive(
    name: &str,
    repo: &PackageList,
    global_recipe: Option<&Recipe>,
    visiting: &mut HashSet<String>,
) {
    if visiting.contains(name) {
        panic!("dependency cycle detected: {}", name);
    }

    let installed_now = load_installed();
    if installed_now.packages.iter().any(|p| p.name == name) {
        println!("{} already installed; skipping", name);
        return;
    }

    let pkg_owned = if let Some(p) = find_in_repo(repo, name) {
        p.clone()
    } else {
        let installed = load_installed();
        installed
            .packages
            .iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| {
                panic!(
                    "package not found in package index or installed.json: {}",
                    name
                )
            })
            .clone()
    };

    visiting.insert(name.to_string());
    for dep in &pkg_owned.dependencies {
        install_recursive(dep, repo, global_recipe, visiting);
    }

    let installed_after = load_installed();
    if installed_after.packages.iter().any(|p| p.name == name) {
        println!("{} installed by dependency step; skipping", name);
        visiting.remove(name);
        return;
    }

    println!("installing {}", name);
    run_recipe(&pkg_owned, global_recipe);
    let _ = Command::new("rm")
        .args(["-rf", &pkg_owned.archive])
        .status();
    visiting.remove(name);
}

fn remove_recursive(dirname: &str, _repo: &PackageList, visiting: &mut HashSet<String>) {
    if visiting.contains(dirname) {
        panic!("dependency cycle detected for dirname {}", dirname);
    }

    let installed_now = load_installed();
    if !installed_now.packages.iter().any(|p| p.dirname == dirname) {
        println!("{} not installed; skipping", dirname);
        return;
    }

    let pkg = find_installed_by_dirname(&installed_now, dirname).unwrap_or_else(|| {
        panic!(
            "installed package with dirname '{}' not found in {}",
            dirname, INSTALLED_FILE
        )
    });

    visiting.insert(dirname.to_string());

    for dep_name in &pkg.dependencies {
        let installed = load_installed();
        let used_by_other = installed
            .packages
            .iter()
            .any(|p| p.dirname != pkg.dirname && p.dependencies.contains(dep_name));
        if used_by_other {
            println!(
                "dependency '{}' is still required by another package; skipping",
                dep_name
            );
            continue;
        }

        if let Some(dep_pkg) = installed.packages.iter().find(|p| p.name == *dep_name) {
            remove_recursive(&dep_pkg.dirname, _repo, visiting);
        } else {
            println!("warning: dependency '{}' not installed; skipping", dep_name);
        }
    }

    let installed_after = load_installed();
    if !installed_after
        .packages
        .iter()
        .any(|p| p.dirname == dirname)
    {
        println!("{} already removed by dependency cleanup", dirname);
        visiting.remove(dirname);
        return;
    }

    println!("removing {}", pkg.name);
    run_remove_script(&pkg.dirname);
    let _ = fs::remove_file(&pkg.archive);

    visiting.remove(dirname);
}

/* ---------- CLI / main ---------- */

fn edit_recipe(target: &str) {
    let file_to_edit = match target {
        "binary" => RECIPE_FILE,
        "source" => SOURCE_FILE,
        "remove" => REMOVE_FILE,
        other => {
            eprintln!("unknown edit target: {} (use 'binary' or 'source')", other);
            std::process::exit(2);
        }
    };

    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let status = Command::new(editor)
        .arg(file_to_edit)
        .status()
        .unwrap_or_else(|e| panic!("failed to launch editor: {}", e));
    if !status.success() {
        eprintln!("editor exited with non-zero status");
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(ref target) = cli.edit {
        edit_recipe(target);
        return;
    }

    let repo = load_packages("https://raw.githubusercontent.com/Arthur4567321/packages-mtr/refs/heads/main/packages.json");
    let global_recipe = load_global_recipe();
    let installed_now = load_installed();

    // handle search first: support either a substring search (--search term)
    // or checking explicit positional package names (program <pkg> --search)
    if cli.search.is_some() {
        let term = cli.search.as_ref().unwrap();
        if cli.packages.is_empty() {
            let matches: Vec<_> = repo
                .packages
                .iter()
                .filter(|p| p.name.contains(term))
                .collect();
            if matches.is_empty() {
                println!("no packages match '{}'", term);
            } else {
                for p in matches {
                    println!("package found: {}", p.name);
                }
            }
        } else {
            for name in cli.packages.clone() {
                let repo_pkgs = repo.packages.iter().find(|p| p.name == name);
                if let Some(p) = repo_pkgs {
                    println!("package exists: {}", p.name);
                } else {
                    println!("package doesn't exist: {}", name);
                }
            }
        }
        return;
    }

    if cli.packages.is_empty() {
        eprintln!("no package names provided");
        std::process::exit(1);
    }

    if cli.remove {
        for name in cli.packages.clone() {
            let target_pkg = installed_now
                .packages
                .iter()
                .find(|p| p.name == name)
                .unwrap_or_else(|| panic!("installed package not found: {}", name));
            let mut visiting = HashSet::new();
            remove_recursive(&target_pkg.dirname, &repo, &mut visiting);
            println!("removed {}", target_pkg.name);
        }
        return;
    }
    if cli.search.is_some() {
        for name in cli.packages.clone(){
            let repo_pkgs = repo
                .packages
                .iter()
                .find(|p| p.name == name);
            if repo_pkgs.is_some(){
		println!("package exists: {}",repo_pkgs.unwrap().name);
		}
	    else{
		println!("package doesn't exist.");
		}
        }
    }

    if cli.update {
        for name in cli.packages.clone() {
            let repo_pkg = repo
                .packages
                .iter()
                .find(|p| p.name == name)
                .unwrap_or_else(|| panic!("package not found in package index: {}", name));

            let installed = load_installed();
            match installed.packages.iter().find(|p| p.name == name) {
                None => {
                    println!("{} not installed — installing", name);
                    let mut visiting = HashSet::new();
                    install_recursive(&name, &repo, global_recipe.as_ref(), &mut visiting);
                }
                Some(installed_pkg) => {
                    let mut visiting = HashSet::new();
                    if installed_pkg.version != repo_pkg.version {
                        println!(
                            "updating {} from {} to {}",
                            name, installed_pkg.version, repo_pkg.version
                        );
                        remove_recursive(&installed_pkg.dirname, &repo, &mut visiting);
                        install_recursive(&name, &repo, global_recipe.as_ref(), &mut visiting);
                    } else {
                        println!("{} is up to date", name);
                    }
                }
            }
        }
        return;
    }

    let mut visiting = HashSet::new();
    for name in cli.packages.clone() {
        install_recursive(&name, &repo, global_recipe.as_ref(), &mut visiting);
    }
}

