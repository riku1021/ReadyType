use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::application::ports::current_input_source_change::{
    CurrentInputSourceChangeMonitor, CurrentInputSourceChangeSink,
};

use super::input_source::WindowsInputSourceGateway;
use crate::application::ports::input_source::InputSourceGateway;

const POLL_INTERVAL: Duration = Duration::from_millis(100);

enum MonitorCommand {
    Stop,
}

#[derive(Default)]
pub struct WindowsCurrentInputSourceChangeMonitor {
    sender: Option<Sender<MonitorCommand>>,
    worker: Option<JoinHandle<()>>,
}

impl CurrentInputSourceChangeMonitor for WindowsCurrentInputSourceChangeMonitor {
    fn start(&mut self, sink: CurrentInputSourceChangeSink) -> Result<(), String> {
        self.stop()?;
        let (sender, receiver) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("readytype-windows-input-source-monitor".to_owned())
            .spawn(move || {
                let gateway = WindowsInputSourceGateway::new();
                let mut previous = gateway.current().ok();
                loop {
                    match receiver.recv_timeout(POLL_INTERVAL) {
                        Ok(MonitorCommand::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                            break
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            let current = gateway.current().ok();
                            if current.is_some() && current != previous {
                                previous = current;
                                sink();
                            }
                        }
                    }
                }
            })
            .map_err(|error| error.to_string())?;
        self.sender = Some(sender);
        self.worker = Some(worker);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(MonitorCommand::Stop);
        }
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| "Windows input source monitor worker panicked".to_owned())?;
        }
        Ok(())
    }
}

impl Drop for WindowsCurrentInputSourceChangeMonitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
