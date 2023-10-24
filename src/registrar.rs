use std::{
    collections::{hash_map::Entry, HashMap},
    path::Path,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    DebounceEventResult, Debouncer, FileIdMap,
};

use crate::{
    error::{WatchError, WatchResult},
    opts::RegisterOpts,
    EventType, FSEvent,
};

pub type DebouncedWatcher = Debouncer<RecommendedWatcher, FileIdMap>;

#[derive(Default)]
pub struct WatcherRegistrar {
    watchers: HashMap<usize, DebouncedWatcher>,
}

impl std::fmt::Debug for WatcherRegistrar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WatcherRegistrar")
            .field(
                "watchers",
                &self
                    .watchers
                    .keys()
                    .map(|k| (k, "DebouncedWatcher<FileIdMap>"))
                    .collect::<HashMap<_, _>>(),
            )
            .finish()
    }
}

impl WatcherRegistrar {
    pub fn unregister_watcher(&mut self, uid: usize) -> WatchResult<()> {
        if self.watchers.remove(&uid).is_none() {
            return Err(WatchError::UidNotFound(uid));
        }
        Ok(())
    }

    pub fn register_watcher<P: AsRef<Path>>(
        &mut self,
        opts: RegisterOpts,
        watch_paths: &[P],
    ) -> WatchResult<Receiver<FSEvent>> {
        if let Entry::Occupied(_) = self.watchers.entry(opts.uid) {
            return Err(WatchError::DuplicateUid(opts.uid));
        }

        let (event_sender, rx) = mpsc::channel();
        let mut watcher = new_debouncer(
            Duration::from_millis(opts.debounce_changes),
            None,
            move |events| {
                Self::send_events(&event_sender, opts.uid, &opts.watch_for, events);
            },
        )?;

        for path in watch_paths {
            watcher
                .watcher()
                .watch(path.as_ref(), RecursiveMode::NonRecursive)?;
            watcher
                .cache()
                .add_root(path.as_ref(), RecursiveMode::NonRecursive);
        }

        self.watchers.insert(opts.uid, watcher);

        Ok(rx)
    }

    fn send_events(
        sender: &Sender<FSEvent>,
        uid: usize,
        watch_for: &[EventType],
        events: DebounceEventResult,
    ) {
        if let Ok(events) = events {
            for event in events {
                let event_kind =
                    EventType::from_notify_event(&event).filter(|typ| watch_for.contains(typ));

                let event_path = event.paths.first().and_then(|path| path.to_str());

                if let Some((event_type, path)) = event_kind.zip(event_path) {
                    sender
                        .send(FSEvent {
                            uid,
                            event_type,
                            path: path.to_string(),
                        })
                        .expect("failed to send event");
                }
            }
        }
    }
}
