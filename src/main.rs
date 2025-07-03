pub mod config;
pub mod helper;
pub mod manager;
pub mod proxy;
pub mod splitted;

use std::path::PathBuf;

use cote::prelude::*;
use manager::{AppContext, Manager};
use rustyline::{error::ReadlineError, Editor};
use tokio::{spawn, sync::mpsc::channel, task::spawn_blocking};

use crate::{
    helper::DeployHelper,
    manager::{Reply, Request},
    proxy::{proxy, Server},
};

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct DeployCli {
    #[arg(value = "history.txt")]
    history: Option<PathBuf>,
}

#[derive(Debug)]
pub enum Message {
    Interrupted,
    Line(String),
    Report(String),
    Request(Request),
}

impl DeployCli {
    pub async fn main(&self) -> color_eyre::Result<()> {
        let user = whoami::username();
        let prompt = format!("♫|{}|>", user);

        let mut ctx = AppContext::default();
        let mut readline = Editor::<DeployHelper, _>::new()?;
        let (
            proxy_cli,
            Server {
                send: proxy_tx,
                recv: mut proxy_rx,
            },
        ) = proxy::<Reply, Request>(32);
        let history = self.history.clone();

        readline.set_helper(Some(DeployHelper::new(proxy_cli)));
        if let Some(path) = &self.history {
            if let Err(e) = readline.load_history(path) {
                eprintln!("WARN! Failed load history file `{}`: {e:?}", path.display());
            }
        }

        let (rl_start_tx, mut rl_start_rx) = channel::<()>(16);
        let (req_server_tx, mut message_rx) = channel(32);
        let readline_tx = req_server_tx.clone();

        // start readline in background
        spawn_blocking(move || {
            let history = history;

            while let Some(()) = rl_start_rx.blocking_recv() {
                let ret = readline.readline(&prompt);

                match ret {
                    Ok(line) => {
                        let line = line.trim().to_string();

                        if !line.is_empty() {
                            readline.add_history_entry(line.clone())?;
                            readline_tx.blocking_send(Message::Line(line))?;
                        }
                    }
                    Err(ReadlineError::Interrupted) => {
                        readline_tx.blocking_send(Message::Interrupted)?;
                        break;
                    }
                    Err(e) => {
                        readline_tx
                            .blocking_send(Message::Report(format!("Got error: {:?}", e)))?;
                    }
                }
            }

            if let Some(path) = &history {
                readline.save_history(path)?;
            }

            Ok::<_, color_eyre::Report>(())
        });

        // start a task process server request
        spawn(async move {
            while let Some(msg) = proxy_rx.recv().await {
                req_server_tx.send(Message::Request(msg)).await?;
            }
            Ok::<_, color_eyre::Report>(())
        });

        // process message
        loop {
            rl_start_tx.send(()).await?;
            if let Some(msg) = message_rx.recv().await {
                match msg {
                    Message::Line(line) => {
                        let splitted = splitted::Splitted::new(&line);
                        let args = splitted.split_args(None).args;

                        if let Err(e) = Manager::invoke_cmd(args, &mut ctx).await {
                            eprintln!("Got error: {e:?}")
                        }
                    }
                    Message::Interrupted => {
                        break;
                    }
                    Message::Report(msg) => {
                        eprintln!("{}", msg);
                    }
                    Message::Request(req) => match req {
                        Request::FetchInstanceId => {
                            proxy_tx
                                .send(Reply::InstanceId(
                                    ctx.insts.iter().map(|v| v.id).collect::<Vec<_>>(),
                                ))
                                .await?;
                        }
                    },
                }
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
