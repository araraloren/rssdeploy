mod kill;
mod list;
mod load;
mod manager;
mod start;

use std::path::PathBuf;

use cote::prelude::*;
use manager::Manager;
use rustyline::{error::ReadlineError, DefaultEditor};

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct Readline {
    #[arg(value = "history.txt")]
    history: PathBuf,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Readline::parse_env()?;
    let user = whoami::username();
    let prompt = format!("♫|{}|>", user);

    let mut rl = DefaultEditor::new()?;
    let mut manager = Manager::default();

    rl.load_history(&args.history)?;
    loop {
        let line = rl.readline(&prompt);

        match line {
            Ok(line) => {
                rl.add_history_entry(line.clone())?;

                let args = line.split_whitespace();

                if let Err(e) = manager.invoke_cmd(&args.collect::<Vec<&str>>()).await {
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
    rl.save_history(&args.history)?;
    Ok(())
}
