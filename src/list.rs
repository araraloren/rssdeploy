use cote::*;

#[derive(Debug, Cote)]
#[cote(width = 50, overload)]
pub struct List {
    /// Instead, list the configuration
    #[arg(alias = "-l")]
    pub local: bool,
}
