use std::sync::mpsc::{self, Receiver, RecvError};
use std::time::Duration;

use crownos_config::{Appearance, Subscription, Theme as ThemeOverrides};

use crate::config::EngineConfig;

/// crownos-settings debounces its own writes by 200 ms, so a slider drag
/// arrives as a burst of edits.
const SETTLE: Duration = Duration::from_millis(250);

const SECTIONS: [&str; 3] = [
    Appearance::SECTION,
    ThemeOverrides::SECTION,
    EngineConfig::SECTION,
];

/// Edits to the appearance, theme and engine settings, from any process.
pub struct SettingsChanges {
    receiver: Receiver<()>,
    _subscriptions: Vec<Subscription>,
}

impl SettingsChanges {
    pub fn subscribe() -> Self {
        let (sender, receiver) = mpsc::channel();
        let subscriptions = SECTIONS
            .iter()
            .map(|section| {
                let sender = sender.clone();
                crownos_config::subscribe(section, move |_| sender.send(()).unwrap_or_default())
            })
            .collect();
        Self {
            receiver,
            _subscriptions: subscriptions,
        }
    }

    /// Blocks until a burst of edits has gone quiet for [`SETTLE`].
    ///
    /// # Errors
    ///
    /// Fails only if the config watcher has shut down.
    pub fn wait(&self) -> Result<(), RecvError> {
        self.receiver.recv()?;
        while self.receiver.recv_timeout(SETTLE).is_ok() {}
        Ok(())
    }
}
