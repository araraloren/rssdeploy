pub mod config;
pub mod manager;

use std::path::PathBuf;

use cote::prelude::*;
use manager::{AppContext, Manager};
use rustyline::{error::ReadlineError, DefaultEditor};

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct DeployCli {
    #[arg(value = "history.txt")]
    history: Option<PathBuf>,
}

impl DeployCli {
    pub async fn main(&self) -> color_eyre::Result<()> {
        let user = whoami::username();
        let prompt = format!("♫|{}|>", user);

        let mut ctx = AppContext::default();
        let mut rl = DefaultEditor::new()?;

        if let Some(path) = &self.history {
            if let Err(e) = rl.load_history(path) {
                eprintln!("WARN! Failed load history file `{}`: {e:?}", path.display());
            }
        }
        loop {
            let line = rl.readline(&prompt);

            match line {
                Ok(line) => {
                    rl.add_history_entry(line.clone())?;

                    if let Err(e) =
                        Manager::invoke_cmd(line.split_whitespace().collect(), &mut ctx).await
                    {
                        eprintln!("Got error: {e:?}")
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    break;
                }
                Err(err) => {
                    eprintln!("Got error: {:?}", err);
                    break;
                }
            }
        }
        if let Some(path) = &self.history {
            rl.save_history(path)?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    DeployCli::parse_env()?.main().await?;

    Ok(())
}
