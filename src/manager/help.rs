use cote::aopt::prelude::Cmd;
use cote::prelude::*;

use super::{kill::Kill, list::List, load::Load, start::Start, AppContext};

#[derive(Debug, Cote)]
#[cote(aborthelp, width = 50, overload, notexit)]
pub struct Help {
    /// Show help message of kill command
    #[cmd()]
    kill: Cmd,

    /// Show help message of list command
    #[cmd()]
    list: Cmd,

    /// Show help message of load command
    #[cmd()]
    load: Cmd,

    /// Show help message of start command
    #[cmd()]
    start: Cmd,
}

impl Help {
    pub async fn invoke_cmd(&self, _ac: &mut AppContext) -> color_eyre::Result<()> {
        if self.kill.0 {
            let parser = Kill::into_parser()?;

            parser.display_help_ctx(Kill::new_help_context().with_name("kill"))?;
        } else if self.list.0 {
            let parser = List::into_parser()?;

            parser.display_help_ctx(List::new_help_context().with_name("list"))?;
        } else if self.load.0 {
            let parser = Load::into_parser()?;

            parser.display_help_ctx(Load::new_help_context().with_name("load"))?;
        } else if self.start.0 {
            let parser = Start::into_parser()?;

            parser.display_help_ctx(Start::new_help_context().with_name("start"))?;
        }

        Ok(())
    }
}
