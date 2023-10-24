pub mod error;
pub mod opts;
pub mod registrar;
pub mod transport;

use error::WatchResult;

use normpath::BasePathBuf;
use opts::{EventType, RegisterOpts};
use registrar::WatcherRegistrar;
use same_file::is_same_file;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

#[derive(Debug, Serialize)]
pub struct FSEvent {
    uid: usize,
    event_type: EventType,
    path: String,
}

#[derive(Debug, Deserialize)]
pub enum InputCmd {
    #[serde(rename = "register")]
    Register(RegisterOpts),
    #[serde(rename = "unregister")]
    Unregister(usize),
}

pub fn handle_cmd(
    line: String,
    writer_tx: Sender<String>,
    registrar: &mut WatcherRegistrar,
) -> WatchResult<()> {
    match serde_json::from_str(&line)? {
        InputCmd::Register(opts) => {
            let paths = get_paths(writer_tx.clone(), &opts)?;
            let event_rx = registrar.register_watcher(opts, &paths)?;
            while let Ok(event) = event_rx.recv() {
                writer_tx.send(serde_json::to_string(&event)?)?;
            }
        }
        InputCmd::Unregister(uid) => {
            registrar.unregister_watcher(uid)?;
        }
    };
    Ok(())
}

fn get_paths(writer_tx: Sender<String>, opts: &RegisterOpts) -> WatchResult<Vec<BasePathBuf>> {
    let (mut paths, pattern_errs) = opts.patterns_to_paths()?;
    writer_tx.send(String::from("invalid patterns..."))?;
    pattern_errs
        .iter()
        .try_for_each(|e| writer_tx.send(e.to_string()))?;

    let (ignores, ignore_errs) = opts.ignores_to_paths()?;
    writer_tx.send(String::from("\ninvalid ignores..."))?;
    ignore_errs
        .iter()
        .try_for_each(|e| writer_tx.send(e.to_string()))?;

    paths.retain(|path| {
        !ignores
            .iter()
            .any(|ignore| is_same_file(path, ignore).unwrap_or(false))
    });

    writer_tx.send(String::from("\nwatching paths..."))?;
    paths
        .iter()
        .try_for_each(|path| writer_tx.send(path.as_path().display().to_string()))?;

    Ok(paths)
}
