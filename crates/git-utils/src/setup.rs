use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    fn as_str(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
    }
}

#[derive(Args)]
pub struct Setup {
    /// Specify shell (bash, zsh, fish)
    #[arg(long)]
    shell: Option<Shell>,

    /// Print configuration snippet for the given shell (bash, zsh, fish)
    #[arg(long, value_name = "SHELL")]
    print: Option<Shell>,

    /// Print gitconfig settings
    #[arg(long)]
    gitconfig: bool,

    /// Uninstall git-utils setup
    #[arg(long)]
    uninstall: bool,
}

const ENV_SH_TEMPLATE: &str = r#"# git-utils environment setup (bash/zsh)
export GIT_REPOS_ROOT="${GIT_REPOS_ROOT:-$HOME/src}"

# Shell function for repository switching
grs() {
    local repo=$(git-repos ls 2>/dev/null | fzf \
        --preview 'git -C $GIT_REPOS_ROOT/{} log -1 --format="%cr%n%s" 2>/dev/null' \
        --preview-window=right:50%:wrap \
        --height=100%)

    if [ -n "$repo" ]; then
        cd "$GIT_REPOS_ROOT/$repo"
    fi
}
"#;

const ENV_FISH_TEMPLATE: &str = r#"# git-utils environment setup (fish)
set -gx GIT_REPOS_ROOT (test -n "$GIT_REPOS_ROOT"; and echo $GIT_REPOS_ROOT; or echo "$HOME/src")

# Shell function for repository switching
function grs
    set result (git-repos ls 2>/dev/null | fzf \
        --preview 'git -C $GIT_REPOS_ROOT/{} log -1 --format="%cr%n%s" 2>/dev/null' \
        --preview-window=right:50%:wrap \
        --height=100%)

    if test -n "$result"
        cd "$GIT_REPOS_ROOT/$result"
        commandline -f repaint
    end
end
"#;

const GITCONFIG_TEMPLATE: &str = r#"# git-utils recommended settings
[git-repos]
    root = ~/src
    prefer-ssh = true

[git-branch-delete]
    base = main

# Git aliases
[alias]
    bs = !git-branch-switch
    bd = !git-branch-delete
    repos = !git-repos
    pr-merged = !git-pr-merged
"#;

impl Setup {
    pub fn execute(&self) -> Result<()> {
        if self.uninstall {
            return self.uninstall_setup();
        }

        if self.gitconfig {
            println!("{}", GITCONFIG_TEMPLATE);
            return Ok(());
        }

        if let Some(shell) = self.print {
            return self.print_config(shell);
        }

        // Auto setup
        self.auto_setup()
    }

    fn auto_setup(&self) -> Result<()> {
        let git_utils_dir = Self::get_git_utils_dir()?;

        // Create directory if it doesn't exist
        if !git_utils_dir.exists() {
            fs::create_dir_all(&git_utils_dir)?;
            println!("Created directory: {}", git_utils_dir.display());
        }

        // Write env files
        let env_sh = git_utils_dir.join("env.sh");
        let env_fish = git_utils_dir.join("env.fish");
        let env_sh_example = git_utils_dir.join("env.sh.example");
        let env_fish_example = git_utils_dir.join("env.fish.example");

        // Always write templates to .example files
        fs::write(&env_sh_example, ENV_SH_TEMPLATE)?;
        fs::write(&env_fish_example, ENV_FISH_TEMPLATE)?;

        println!("Updated template files:");
        println!("  {}", env_sh_example.display());
        println!("  {}", env_fish_example.display());

        // Only create actual env files on first install
        let mut created_files: Vec<PathBuf> = Vec::new();
        let mut existing_files: Vec<PathBuf> = Vec::new();

        if !env_sh.exists() {
            fs::write(&env_sh, ENV_SH_TEMPLATE)?;
            created_files.push(env_sh.clone());
        } else {
            existing_files.push(env_sh.clone());
        }

        if !env_fish.exists() {
            fs::write(&env_fish, ENV_FISH_TEMPLATE)?;
            created_files.push(env_fish.clone());
        } else {
            existing_files.push(env_fish.clone());
        }

        if !created_files.is_empty() {
            println!("\nCreated environment files:");
            for file in &created_files {
                println!("  {}", file.display());
            }
        }

        if !existing_files.is_empty() {
            println!("\nExisting files preserved (not overwritten):");
            for file in &existing_files {
                println!("  {}", file.display());
            }
            println!("\nTo update your env files with new templates, compare with .example files:");
            println!(
                "  git diff --no-index {} {}",
                env_sh.display(),
                env_sh_example.display()
            );
            println!(
                "  git diff --no-index {} {}",
                env_fish.display(),
                env_fish_example.display()
            );
        }

        // Detect shell and add source line
        let shell = if let Some(s) = &self.shell {
            *s
        } else {
            Self::detect_shell()?
        };

        self.add_source_line(shell)?;

        println!("\nSetup complete!");
        let rc_path = match shell {
            Shell::Fish => "config/fish/config.fish".to_string(),
            Shell::Bash => "bashrc".to_string(),
            Shell::Zsh => "zshrc".to_string(),
        };
        println!("Please restart your shell or run: source ~/.{}", rc_path);

        Ok(())
    }

    fn add_source_line(&self, shell: Shell) -> Result<()> {
        let (rc_file, source_line) = match shell {
            Shell::Bash => (
                Self::get_home_dir()?.join(".bashrc"),
                "[ -f ~/.git-utils/env.sh ] && source ~/.git-utils/env.sh\n",
            ),
            Shell::Zsh => (
                Self::get_home_dir()?.join(".zshrc"),
                "[ -f ~/.git-utils/env.sh ] && source ~/.git-utils/env.sh\n",
            ),
            Shell::Fish => (
                Self::get_home_dir()?.join(".config/fish/config.fish"),
                "test -f ~/.git-utils/env.fish && source ~/.git-utils/env.fish\n",
            ),
        };

        // Create parent directory for fish config if needed
        if matches!(shell, Shell::Fish) {
            if let Some(parent) = rc_file.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        // Check if already added
        if rc_file.exists() {
            let file = fs::File::open(&rc_file)?;
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                if line.contains("git-utils/env") {
                    println!("Source line already exists in {}", rc_file.display());
                    return Ok(());
                }
            }
        }

        // Append source line
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_file)?;

        file.write_all(b"\n# git-utils\n")?;
        file.write_all(source_line.as_bytes())?;

        println!("Added source line to {}", rc_file.display());

        Ok(())
    }

    fn print_config(&self, shell: Shell) -> Result<()> {
        match shell {
            Shell::Bash | Shell::Zsh => {
                println!("# Add this to your ~/.{}rc:", shell.as_str());
                println!("[ -f ~/.git-utils/env.sh ] && source ~/.git-utils/env.sh");
            }
            Shell::Fish => {
                println!("# Add this to your ~/.config/fish/config.fish:");
                println!("test -f ~/.git-utils/env.fish && source ~/.git-utils/env.fish");
            }
        }
        Ok(())
    }

    fn uninstall_setup(&self) -> Result<()> {
        let git_utils_dir = Self::get_git_utils_dir()?;

        // Remove source lines from rc files
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            let rc_file = match shell {
                Shell::Bash => Self::get_home_dir()?.join(".bashrc"),
                Shell::Zsh => Self::get_home_dir()?.join(".zshrc"),
                Shell::Fish => Self::get_home_dir()?.join(".config/fish/config.fish"),
            };

            if rc_file.exists() {
                Self::remove_source_lines(&rc_file)?;
                println!("Removed source line from {}", rc_file.display());
            }
        }

        // Remove git-utils directory
        if git_utils_dir.exists() {
            fs::remove_dir_all(&git_utils_dir)?;
            println!("Removed directory: {}", git_utils_dir.display());
        }

        println!("Uninstall complete!");

        Ok(())
    }

    fn remove_source_lines(rc_file: &PathBuf) -> Result<()> {
        let content = fs::read_to_string(rc_file)?;
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();
        let mut skip_next = false;

        for line in lines {
            if line.contains("# git-utils") {
                skip_next = true;
                continue;
            }
            if skip_next && line.contains("git-utils/env") {
                skip_next = false;
                continue;
            }
            skip_next = false;
            new_lines.push(line);
        }

        fs::write(rc_file, new_lines.join("\n") + "\n")?;
        Ok(())
    }

    fn detect_shell() -> Result<Shell> {
        if let Ok(shell_path) = std::env::var("SHELL") {
            if let Some(shell_name) = shell_path.split('/').next_back() {
                return match shell_name {
                    "bash" => Ok(Shell::Bash),
                    "zsh" => Ok(Shell::Zsh),
                    "fish" => Ok(Shell::Fish),
                    _ => Err(anyhow!(
                        "Unsupported shell: {}. Please specify with --shell (bash, zsh, fish)",
                        shell_name
                    )),
                };
            }
        }
        Err(anyhow!(
            "Could not detect shell. Please specify with --shell (bash, zsh, fish)"
        ))
    }

    fn get_home_dir() -> Result<PathBuf> {
        dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))
    }

    fn get_git_utils_dir() -> Result<PathBuf> {
        Ok(Self::get_home_dir()?.join(".git-utils"))
    }
}
