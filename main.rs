use std::collections::HashMap;
use std::process::Command;

// ============ Build ============

pub struct Build {
    name: String,
    description: String,
    steps: Vec<(String, String)>,
    env: HashMap<String, String>,
    output_dir: String,
    depends_on: Vec<String>,
}

impl Build {
    pub fn new(name: &str) -> Self {
        Build {
            name: name.to_string(),
            description: String::new(),
            steps: Vec::new(),
            env: HashMap::new(),
            output_dir: format!("./target/{}", name),
            depends_on: Vec::new(),
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn step(mut self, step_name: &str, cmd: &str) -> Self {
        self.steps.push((step_name.to_string(), cmd.to_string()));
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    pub fn output_dir(mut self, dir: &str) -> Self {
        self.output_dir = dir.to_string();
        self
    }

    pub fn depends_on(mut self, deps: Vec<&str>) -> Self {
        self.depends_on = deps.iter().map(|d| d.to_string()).collect();
        self
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Building: {} - {}", self.name, self.description);
        
        std::fs::create_dir_all(&self.output_dir)?;

        for (step_name, cmd) in &self.steps {
            println!("  Step: {}", step_name);
            
            let mut child = Command::new("sh")
                .arg("-c")
                .arg(cmd);

            for (k, v) in &self.env {
                child.env(k, v);
            }

            child.env("OUT", &self.output_dir);

            let status = child.status()?;
            if !status.success() {
                return Err(format!("Build step '{}' failed", step_name).into());
            }
        }

        println!("  ✓ {}\n", self.name);
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn depends_on_list(&self) -> &[String] {
        &self.depends_on
    }
}

// ============ Shell ============

pub struct Shell {
    name: String,
    env: HashMap<String, String>,
    packages: Vec<String>,
    hooks: Vec<String>,
    depends_on: Vec<String>,
}

impl Shell {
    pub fn new(name: &str) -> Self {
        Shell {
            name: name.to_string(),
            env: HashMap::new(),
            packages: Vec::new(),
            hooks: Vec::new(),
            depends_on: Vec::new(),
        }
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    pub fn packages(mut self, pkgs: Vec<&str>) -> Self {
        self.packages = pkgs.iter().map(|p| p.to_string()).collect();
        self
    }

    pub fn hook(mut self, cmd: &str) -> Self {
        self.hooks.push(cmd.to_string());
        self
    }

    pub fn depends_on(mut self, deps: Vec<&str>) -> Self {
        self.depends_on = deps.iter().map(|d| d.to_string()).collect();
        self
    }

    pub fn enter(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Entering shell: {}", self.name);
        
        let mut script = String::new();
        
        // Set environment variables
        for (k, v) in &self.env {
            script.push_str(&format!("export {}={}\n", k, v));
        }

        // Run hooks
        for hook in &self.hooks {
            script.push_str(&format!("{}\n", hook));
        }

        // Launch shell
        let shell_cmd = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        script.push_str(&format!("exec {}\n", shell_cmd));

        let status = Command::new("sh")
            .arg("-c")
            .arg(&script)
            .status()?;

        if !status.success() {
            return Err("Shell exited with error".into());
        }

        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn depends_on_list(&self) -> &[String] {
        &self.depends_on
    }
}

// ============ Project ============

pub struct Project {
    name: String,
    builds: HashMap<String, Build>,
    shells: HashMap<String, Shell>,
}

impl Project {
    pub fn new(name: &str) -> Self {
        Project {
            name: name.to_string(),
            builds: HashMap::new(),
            shells: HashMap::new(),
        }
    }

    pub fn with_build(mut self, build: Build) -> Self {
        self.builds.insert(build.name().to_string(), build);
        self
    }

    pub fn with_shell(mut self, shell: Shell) -> Self {
        self.shells.insert(shell.name().to_string(), shell);
        self
    }

    pub fn build(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let build = self.builds.get(name)
            .ok_or_else(|| format!("Build '{}' not found", name))?;

        // Build dependencies first
        for dep in build.depends_on_list() {
            if let Some(dep_build) = self.builds.get(dep) {
                println!("Building dependency: {}", dep);
                dep_build.execute()?;
            }
        }

        build.execute()?;
        Ok(())
    }

    pub fn shell(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let shell = self.shells.get(name)
            .ok_or_else(|| format!("Shell '{}' not found", name))?;

        // Build dependencies first
        for dep in shell.depends_on_list() {
            if let Some(dep_build) = self.builds.get(dep) {
                println!("Building dependency: {}", dep);
                dep_build.execute()?;
            }
        }

        shell.enter()?;
        Ok(())
    }

    pub fn display(&self) {
        println!("\n=== Project: {} ===\n", self.name);

        if !self.builds.is_empty() {
            println!("Builds:");
            for (name, build) in &self.builds {
                println!("  {} - {}", name, build.description);
            }
        }

        if !self.shells.is_empty() {
            println!("\nShells:");
            for name in self.shells.keys() {
                println!("  {}", name);
            }
        }
        println!();
    }
}
