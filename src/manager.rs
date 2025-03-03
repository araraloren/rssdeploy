mod help;
mod kill;
mod list;
mod load;
mod start;

use cote::prelude::*;
use help::Help;
use tokio::process::Child;

use crate::config::DeployConfig;

use kill::Kill;
use list::List;
use load::Load;
use start::Start;

#[derive(Debug)]
pub struct SsInstance {
    pub id: usize,

    pub ss: Child,

    pub kcp: Option<Child>,
}

#[derive(Debug, Default)]
pub struct AppContext {
    pub cfgs: Vec<DeployConfig>,

    pub insts: Vec<SsInstance>,
}

#[derive(Debug, Default, Cote)]
#[cote(aborthelp, width = 50, overload, notexit)]
pub struct Manager {
    /// List the instances or configurations
    #[sub(alias = "ls")]
    list: Option<List>,

    /// Kill instance by id
    #[sub()]
    kill: Option<Kill>,

    /// Load deploy configurations from *.json
    #[sub(alias = "ld")]
    load: Option<Load>,

    /// Start instance by id or configuration path
    #[sub(alias = "st")]
    start: Option<Start>,

    /// Display the help of given command
    #[sub()]
    help: Option<Help>,
}

impl Manager {
    pub async fn invoke_cmd(args: Vec<&str>, ac: &mut AppContext) -> color_eyre::Result<()> {
        let args: Vec<_> = std::iter::once("app").chain(args.into_iter()).collect();
        let manager = Manager::parse(Args::from(args))?;

        if let Some(list) = manager.list {
            list.invoke_cmd(ac).await?;
        } else if let Some(kill) = manager.kill {
            kill.invoke_cmd(ac).await?;
        } else if let Some(load) = manager.load {
            load.invoke_cmd(ac).await?;
        } else if let Some(start) = manager.start {
            start.invoke_cmd(ac).await?;
        } else if let Some(help) = manager.help {
            help.invoke_cmd(ac).await?;
        }

        Ok(())
    }
}
