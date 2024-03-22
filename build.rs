include!("src/init/cli.rs");
use clap::CommandFactory;
use clap_complete_command::Shell::{Bash, Elvish, Fig, Fish, PowerShell, Zsh};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = ::std::env::var_os("OUT_DIR").expect("OUT_DIR not found.");
    let out_dir = std::path::PathBuf::from(out_dir);

    for shell in [Bash, Elvish, Fig, Fish, PowerShell, Zsh] {
        shell.generate_to(&mut Cli::command(), &out_dir)?;
    }

    let man = clap_mangen::Man::new(Cli::command());
    let mut buffer: Vec<u8> = Vec::default();
    man.render(&mut buffer)?;
    std::fs::write(out_dir.join("archiver.1"), buffer)?;

    Ok(())
}
