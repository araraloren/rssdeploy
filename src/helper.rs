use std::ffi::OsString;
use std::marker::PhantomData;

use cote::shell::shell::Complete;
use cote::shell::shell::Shell;
use cote::shell::CompletionManager;
use cote::shell::Context;
use rustyline::completion::Completer;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::Completer;
use rustyline::Helper;
use rustyline::Highlighter;
use rustyline::Hinter;
use rustyline::Validator;

use crate::manager::Manager;
use crate::splitted::Splitted;

#[derive(Default, Helper, Completer, Highlighter, Hinter, Validator)]
pub struct DeployHelper {
    #[rustyline(Completer)]
    completer: DeployCompleter,
    #[rustyline(Hinter)]
    hinter: (),
    #[rustyline(Highlighter)]
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
}

#[derive(Debug, Default)]
pub struct DeployCompleter {}

impl Completer for DeployCompleter {
    type Candidate = String;

    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let splitted = Splitted::new(line);
        let mut replace = 0;
        let mut index = splitted.len();

        for (id, slice) in splitted.iter().enumerate() {
            let len = slice.dat.len();

            if pos >= replace && pos <= replace + len {
                index = id;
                break;
            }
            replace += len;
        }

        // println!("pos={pos} `{line}`",);
        // println!(" --->  {:#?}", splits);
        // println!("index = {index}, replace = {replace}");

        let crate::splitted::Args { cword, args } = splitted.split_args(Some(index));
        let args: Vec<_> = std::iter::once(OsString::from("rssdeploy"))
            .chain(args.iter().map(OsString::from))
            .collect();
        let def = OsString::default();

        if let Ok(parser) = Manager::into_parser() {
            // +1 for extra "rssdeploy"
            let cword = cword.map(|v| v + 1).unwrap_or(args.len());
            let curr = args.get(cword).unwrap_or(&def);
            let prev = args.get(cword - 1).unwrap_or(&def);

            // println!("curr = {curr:?}");
            // println!("curr = {prev:?}");
            // println!("curr = {cword:?}");
            // println!("curr = {args:?}");

            let mut context = Context::new(&args, curr, prev, cword);
            let mut manager = CompletionManager::new(parser);
            let mut shell = DeployShell::new();

            let _ = Manager::inject_completion_values(&mut manager);

            if manager.complete(&mut shell, &mut context).is_ok() {
                return Ok(((replace + 1).min(pos), shell.w));
            }
        }

        <()>::complete(&(), line, pos, ctx)
    }
}

// pub fn split_to_args_without(val: Splitted<'_>, pos: Option<usize>) ->  {
//     let mut cword = None;

//     for (id, slice) in splits.iter().enumerate() {
//         let dat = slice.dat.trim();

//         if id == index {
//             cword = Some(args.len());
//             args.push(OsString::from(dat));
//         } else if !dat.is_empty() {
//             args.push(OsString::from(dat));
//         }
//     }
// }

pub struct DeployShell<O> {
    w: Vec<String>,
    __marker: PhantomData<O>,
}

impl<O> Default for DeployShell<O> {
    fn default() -> Self {
        Self {
            w: Default::default(),
            __marker: Default::default(),
        }
    }
}

impl<O> DeployShell<O> {
    pub fn new() -> Self {
        Self {
            w: vec![],
            __marker: PhantomData,
        }
    }
}

impl<O> Shell<O, Vec<String>> for DeployShell<O> {
    type Err = cote::Error;

    fn is_avail(&self, _: &str) -> bool {
        true
    }

    fn set_buff(&mut self, w: Vec<String>) {
        self.w = w;
    }

    fn write_cmd(&mut self, name: &str, _: &O) -> Result<(), Self::Err> {
        self.w.push(name.to_string());
        Ok(())
    }

    fn write_opt(&mut self, name: &str, _: &O) -> Result<(), Self::Err> {
        self.w.push(name.to_string());
        Ok(())
    }

    fn write_pos(&mut self, name: &str, _: &O) -> Result<(), Self::Err> {
        self.w.push(name.to_string());
        Ok(())
    }

    fn write_val(&mut self, val: &std::ffi::OsStr, _: &O) -> Result<(), Self::Err> {
        self.w.push(val.display().to_string());
        Ok(())
    }

    fn write_eq(&mut self, _: &str, val: &std::ffi::OsStr, _: &O) -> Result<(), Self::Err> {
        self.w.push(val.display().to_string());
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn take_buff(&mut self) -> Option<Vec<String>> {
        Some(std::mem::take(&mut self.w))
    }
}
