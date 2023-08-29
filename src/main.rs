mod server;

use std::{path::PathBuf, sync::atomic::AtomicBool};

use cote::{*};
use rustyline::{error::ReadlineError, DefaultEditor};

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct Cli {
    #[arg(value = "history.txt")]
    history: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Cli::parse_env()?;
    let user = whoami::username();
    let prompt = format!("♫|{}|>", user);
    let mut rl = DefaultEditor::new()?;

    if rl.load_history(&args.history).is_err() {
        eprintln!("No history found!");
    }
    loop {
        let line = rl.readline(&prompt);

        match line {
            Ok(line) => {
                let args = line.split_whitespace();
                let args = args.collect::<Vec<&str>>();
                let command = args.get(0).cloned();

                match command {
                    Some("start") => {
                        if let Some(start) = process_start(&args)? {
                            dbg!(start);
                        }
                    },
                    _ => {
                        println!("Got unknown commands: {:?}", args);
                    },
                }
            },
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(err) => {
                eprintln!("Got error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Cote)]
#[cote(width = 50, overload)]
pub struct Start {
    /// Set the prefix of ssserver
    #[arg(alias = "-p", value = "/root/.cargo", valid = valid!(|v: &PathBuf| v.exists()))]
    prefix: PathBuf,

    /// Start a kcptun server at 127.0.0.1:{port + 1}
    #[arg(alias = "-k")]
    kcptun: bool,

    /// Set the listen port of kcptun
    #[arg(alias = "-l")]
    listen: Option<u32>,

    /// Set the transform ip address of kcptun
    #[arg(alias = "-s", value = "127.0.0.1")]
    server: String,

    /// Set the transform port of kcptun
    #[arg(alias = "-p")]
    port: Option<u32>,

    /// Set send windows size
    #[arg(value = 2048u32)]
    send: u32,

    /// Set receive windows size
    #[arg(value = 2048u32)]
    recv: u32,

    /// Set mtu value
    #[arg(value = 1400u32)]
    mtu: u32,

    /// Set dscp value
    #[arg(value = 46u32)]
    dscp: u32,

    /// Set datashard value
    #[arg(value = 30u32)]
    datashard: u32,

    /// Set parityshard value
    #[arg(value = 15u32)]
    parityshard: u32,

    /// Set kcptun mode
    #[arg(value = "fast2")]
    mode: String,

    /// Display help message
    #[arg(alias = "-h")]
    help: bool,
}

pub fn process_start<'a>(args: &[&str]) -> color_eyre::Result<Option<Start>> {
    let args = Args::from(args.iter().map(|v|*v));
    let CoteRes { mut policy, mut ret, mut parser } = Start::parse_args(args)?;

    if ret.status() {
        let help = parser.find_val::<bool>("--help")?;

        if *help {
            let help = StartInternalApp { parser: Some(&mut parser), policy: Some(&mut policy) };

            help.display_help()?;
            Ok(None)
        }
        else {
            Ok(Some(Start::try_extract(parser.optset_mut())?))
        }
    }
    else {
        Err(ret.take_failure())?
    }
}