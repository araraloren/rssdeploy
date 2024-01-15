use cote::*;

#[derive(Debug, Cote)]
#[cote(width = 50, overload)]
pub struct Kill {
    /// Kill all shadowsocks instance
    pub all: bool,

    /// Kill given shadowsocks instance
    #[arg(alias = "-i", value = 0usize)]
    pub index: Option<usize>,
}
