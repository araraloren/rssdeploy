use std::env::current_dir;
use std::ffi::OsString;
use std::fs::read_dir;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use cote::prelude::Args;
use cote::prelude::PolicyParser;
use cote::prelude::SetValueFindExt;
use cote::shell::shell::Complete;
use cote::shell::shell::Shell;
use cote::shell::value::repeat_values;
use cote::shell::value::Values;
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
use crate::manager::Reply;
use crate::manager::Request;
use crate::proxy::Client;
use crate::splitted::Splitted;

#[derive(Helper, Completer, Highlighter, Hinter, Validator)]
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

impl DeployHelper {
    pub fn new(proxy: Client<Reply, Request>) -> Self {
        Self {
            completer: DeployCompleter {
                proxy: Arc::new(Mutex::new(proxy)),
            },
            hinter: (),
            highlighter: MatchingBracketHighlighter::default(),
            validator: MatchingBracketValidator::default(),
        }
    }
}

#[derive(Debug)]
pub struct DeployCompleter {
    proxy: Arc<Mutex<Client<Reply, Request>>>,
}

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

        if let Ok(mut parser) = Manager::into_parser() {
            // +1 for extra "rssdeploy"
            let cword = cword.map(|v| v + 1).unwrap_or(args.len());
            let curr = args.get(cword).unwrap_or(&def);
            let prev = args.get(cword - 1).unwrap_or(&def);

            // println!("curr = {curr:?}");
            // println!("curr = {prev:?}");
            // println!("curr = {cword:?}");
            // println!("curr = {args:?}");
            let mut policy = Manager::into_policy().with_prepolicy(true);

            // process args before completion
            let _ = parser.parse_policy(Args::from(&args), &mut policy);

            let start = parser.take_val::<crate::manager::Start>("start").ok();
            let start_user_config = start.and_then(|v| v.config.map(|v| v.display().to_string()));

            let load = parser.take_val::<crate::manager::Load>("load").ok();
            let load_user_config = load.and_then(|v| v.config);

            let mut context = Context::new(&args, curr, prev, cword);
            let mut manager = CompletionManager::new(parser);
            let mut shell = DeployShell::new();

            let _ = Manager::inject_completion_values(&mut manager);

            // set values of load configurations
            if let Ok(load) = manager.find_manager_mut("load") {
                if let Ok(config_uid) = load.parser().find_uid("--config") {
                    load.set_values(config_uid, load_json_completion(load_user_config));
                }
            }

            // set values of kill id
            if let Ok(kill) = manager.find_manager_mut("kill") {
                let proxy = self.proxy.clone();
                let idlist = std::thread::spawn(move || {
                    proxy.lock().unwrap().req_sync(Request::FetchInstanceId)
                })
                .join()
                .unwrap();

                if let Ok(Reply::InstanceId(ids)) = idlist {
                    if let Ok(index_uid) = kill.parser().find_uid("--id") {
                        kill.set_values(
                            index_uid,
                            ids.into_iter()
                                .map(|v| OsString::from(v.to_string()))
                                .collect::<Vec<_>>(),
                        );
                    }
                }
            }

            // set values of start index
            if let Ok(start) = manager.find_manager_mut("start") {
                let proxy = self.proxy.clone();
                let indexlist = std::thread::spawn(move || {
                    proxy.lock().unwrap().req_sync(Request::FetchTaskIndex)
                })
                .join()
                .unwrap();

                if let Ok(Reply::InstanceId(indices)) = indexlist {
                    if let Ok(index_uid) = start.parser().find_uid("index") {
                        start.set_values(
                            index_uid,
                            indices
                                .into_iter()
                                .map(|v| OsString::from(v.to_string()))
                                .collect::<Vec<_>>(),
                        );
                    }
                }
                if let Ok(config_uid) = start.parser().find_uid("--config") {
                    start.set_values(config_uid, json_completion(start_user_config));
                }
            }

            // if current is empty string, complete at next index
            if curr.is_empty() {
                replace += 1;
            }

            if manager.complete(&mut shell, &mut context).is_ok() {
                return Ok(((replace).min(pos), shell.w));
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

pub fn json_completion<O>(val: Option<String>) -> impl Values<O, Err = cote::Error> {
    file_completion(vec![], val, |v| {
        v.extension().and_then(|v| v.to_str()) == Some("json")
    })
}

pub fn load_json_completion<O>(val: Option<String>) -> impl Values<O, Err = cote::Error> {
    file_completion(
        vec![OsString::from(crate::manager::DEFAULT_CONFIG)],
        val,
        |v| v.extension().and_then(|v| v.to_str()) == Some("json"),
    )
}

pub fn file_completion<O>(
    init: Vec<OsString>,
    dir: Option<String>,
    filter: impl Fn(&Path) -> bool + 'static,
) -> impl Values<O, Err = cote::Error> {
    repeat_values(move |_| {
        let mut vals = init.clone();

        // search .json in current working directory
        if let Some(paths) = dir.as_ref().and_then(|v| complete_all(v).ok()) {
            vals.extend(paths.into_iter().map(OsString::from));
        } else if let Ok(read_dir) = current_dir().and_then(std::fs::read_dir) {
            for entry in read_dir {
                if let Ok(path) = entry.map(|v| v.path()) {
                    if filter(&path) {
                        if let Some(filename) = path.file_name() {
                            let filename = filename.to_os_string();

                            if !vals.contains(&filename) {
                                vals.push(filename);
                            }
                        }
                    }
                }
            }
        }

        Ok(vals)
    })
}

pub fn complete_all(val: &str) -> color_eyre::Result<Vec<String>> {
    complete_path_with(val, |_| true)
}

pub fn complete_files(val: &str) -> color_eyre::Result<Vec<String>> {
    complete_path_with(val, |v| v.is_file())
}

pub fn complete_path_with(
    val: &str,
    filter: impl Fn(&Path) -> bool,
) -> color_eyre::Result<Vec<String>> {
    let mut rets = vec![];
    let dst = Path::new(val);

    if dst.exists() && dst.is_dir() {
        let read_dir = std::fs::read_dir(dst)?;

        for entry in read_dir {
            if let Ok(path) = entry.map(|v| v.path()) {
                if filter(&path) {
                    rets.push(path.display().to_string());
                }
            }
        }
    } else if !dst.exists() {
        if let Some(left) = dst.parent() {
            let path = left.display().to_string();

            if let Some(right) = dst.file_name() {
                let prefix = right.as_encoded_bytes();
                let expand = shellexpand::full(&path)?;
                let expand = Path::new(expand.as_ref());

                if expand.exists() && expand.is_dir() {
                    for entry in read_dir(expand)?.flatten() {
                        let path = entry.path();

                        if filter(&path) {
                            if let Some(filename) = path.file_name() {
                                if filename.as_encoded_bytes().starts_with(prefix) {
                                    rets.push(left.join(filename).display().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    //println!("compelte {val} -> {rets:?}");

    Ok(rets)
}
