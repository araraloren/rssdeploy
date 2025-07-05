pub mod config;
pub mod helper;
pub mod manager;
pub mod proxy;
pub mod splitted;

use std::path::PathBuf;

use cote::prelude::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use tokio::spawn;
use tokio::sync::mpsc::channel;
use tokio::task::spawn_blocking;

use helper::DeployHelper;
use manager::AppContext;
use manager::Manager;
use manager::Reply;
use manager::Request;
use proxy::proxy;
use proxy::Server;

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
        let (
            proxy_cli,
            Server {
                send: proxy_tx,
                recv: mut proxy_rx,
            },
        ) = proxy::<Reply, Request>(32);

        let mut ctx = AppContext::default();
        let mut readline = Editor::<DeployHelper, _>::new()?;

        let history = self.history.clone();

        let (rl_start_tx, mut rl_start_rx) = channel::<()>(16);
        let (req_server_tx, mut message_rx) = channel(32);
        let readline_tx = req_server_tx.clone();

        // start readline in background
        let background_rl_handler = spawn_blocking(move || {
            let user = whoami::username();
            let prompt = format!("♫|{}|>", user);

            readline.set_helper(Some(DeployHelper::new(proxy_cli)));
            if let Some(path) = &history {
                if !path.exists() {
                    // create if file not exists
                    let _ = std::fs::File::create(path)?;
                }
                if let Err(e) = readline.load_history(path) {
                    eprintln!("WARN! Failed load history file `{}`: {e:?}", path.display());
                }
            }
            while let Some(()) = rl_start_rx.blocking_recv() {
                let ret = readline.readline(&prompt);

                match ret {
                    Ok(line) => {
                        let line = line.trim().to_string();

                        if !line.is_empty() {
                            readline.add_history_entry(line.clone())?;
                        }
                        readline_tx.blocking_send(Message::Line(line))?;
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

        let mut ready_readline = true;

        // process message
        loop {
            if ready_readline {
                // start readline
                rl_start_tx.send(()).await?;
                ready_readline = false;
            }
            if let Some(msg) = message_rx.recv().await {
                match msg {
                    Message::Line(line) => {
                        let splitted = splitted::Splitted::new(&line);
                        let args = splitted.split_args(None).args;

                        if let Err(e) = Manager::invoke_cmd(args, &mut ctx).await {
                            eprintln!("Got error: {e:?}")
                        }
                        ready_readline = true;
                    }
                    Message::Interrupted => {
                        break;
                    }
                    Message::Report(msg) => {
                        ready_readline = true;
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

        drop(rl_start_tx);
        background_rl_handler.await??;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    DeployCli::parse_env()?.main().await?;

    Ok(())
}
