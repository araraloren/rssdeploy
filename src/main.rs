pub mod config;
pub mod helper;
pub mod manager;
pub mod proxy;
pub mod splitted;

use std::path::PathBuf;

use cote::prelude::*;
use manager::{AppContext, Manager};
use rustyline::{error::ReadlineError, Editor};
use tokio::{runtime::Handle, select, sync::mpsc::channel};

use crate::{
    helper::DeployHelper,
    manager::{Reply, Request},
    proxy::proxy,
};

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct DeployCli {
    #[arg(value = "history.txt")]
    history: Option<PathBuf>,
}

#[derive(Debug)]
pub enum Readline {
    Interrupted,
    Line(String),
    Report(String),
}

impl DeployCli {
    pub async fn main(&self) -> color_eyre::Result<()> {
        let user = whoami::username();
        let prompt = format!("♫|{}|>", user);

        let mut ctx = AppContext::default();
        let mut rl = Editor::<DeployHelper, _>::new()?;
        let (proxy_cli, mut proxy) = proxy::<Reply, Request>(32);
        let history = self.history.clone();
        let handle = Handle::current();

        rl.set_helper(Some(DeployHelper::new(proxy_cli)));
        if let Some(path) = &self.history {
            if let Err(e) = rl.load_history(path) {
                eprintln!("WARN! Failed load history file `{}`: {e:?}", path.display());
            }
        }

        let (send, mut recv) = channel(32);

        // start readline in background
        std::thread::spawn(move || {
            handle.block_on(async move {
                let mut rl = rl;
                let history = history;

                loop {
                    let ret = rl.readline(&prompt);

                    match ret {
                        Ok(line) => {
                            rl.add_history_entry(line.clone())?;
                            send.send(Readline::Line(line)).await?;
                        }
                        Err(ReadlineError::Interrupted) => {
                            send.send(Readline::Interrupted).await?;
                        }
                        Err(e) => {
                            send.send(Readline::Report(format!("Got error: {:?}", e)))
                                .await?;
                        }
                    }
                    if let Some(path) = &history {
                        rl.save_history(path)?;
                    }
                }

                #[allow(unreachable_code)]
                Ok::<_, color_eyre::Report>(())
            })?;

            Ok::<_, color_eyre::Report>(())
        });

        // process line
        let process_line = async |line: String, ctx: &mut AppContext| -> color_eyre::Result<()> {
            let splitted = splitted::Splitted::new(&line);
            let args = splitted.split_args(None).args;

            if let Err(e) = Manager::invoke_cmd(args, ctx).await {
                eprintln!("Got error: {e:?}")
            }
            Ok(())
        };

        // process msg
        let process_msg = async |req: Request, ctx: &AppContext| -> color_eyre::Result<Reply> {
            match req {
                Request::FetchInstanceId => Ok(Reply::InstanceId(
                    ctx.insts.iter().map(|v| v.id).collect::<Vec<_>>(),
                )),
            }
        };

        loop {
            select! {
                Some(msg) = recv.recv() => {
                    match msg {
                    Readline::Line(line) => process_line(line, &mut ctx).await?,
                    Readline::Interrupted => {
                        break;
                    },
                    Readline::Report(msg) => {
                        eprintln!("{}", msg);
                    },
                }
                },
                Some(msg) = proxy.recv.recv() => {
                    let reply = process_msg(msg, &ctx).await?;

                    proxy.send.send(reply).await?;
                },
                else => {},
            }
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
