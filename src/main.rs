mod kill;
mod list;
mod load;
mod manager;
mod start;

use std::path::PathBuf;

use cote::*;
use manager::Manager;
use rustyline::{error::ReadlineError, DefaultEditor};

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct Readline {
    #[arg(value = "history.txt")]
    history: PathBuf,
}

fn main() -> color_eyre::Result<()> {
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

                if let Err(e) = manager.invoke_cmd(&args.collect::<Vec<&str>>()) {
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

#[macro_export]
macro_rules! ss_display_help {
    ($type:ty, $internal:tt) => {{
        let mut parser = <$type>::into_parser()?;
        let mut policy = <$type>::into_policy();
        let internal = $internal {
            parser: Some(&mut parser),
            policy: Some(&mut policy),
        };

        internal.display_help()?;
    }};
}
