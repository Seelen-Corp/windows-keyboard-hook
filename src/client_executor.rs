use std::sync::{Arc, LazyLock};

use crossbeam_channel::{Receiver, Sender};

use crate::log_on_dev;

static CLIENT_CHANNEL: LazyLock<(Sender<ClientAction>, Receiver<ClientAction>)> =
    LazyLock::new(crossbeam_channel::unbounded);

/// Enum representing client actions, which need to run
/// on a separated thead to avoid deadlocks.
enum ClientAction {
    Call(Arc<dyn Fn() + Send + Sync>),
    Stop,
}

impl ClientAction {
    fn emit(self) {
        if CLIENT_CHANNEL.0.send(self).is_err() {
            log_on_dev!("Failed to send client action");
        }
    }

    fn reciever() -> Receiver<ClientAction> {
        CLIENT_CHANNEL.1.clone()
    }
}

pub(crate) fn start_executor_thread() {
    std::thread::spawn(|| {
        while let Ok(event) = ClientAction::reciever().recv() {
            match event {
                ClientAction::Call(cb) => cb(),
                ClientAction::Stop => break,
            }
        }
    });
}

pub(crate) fn run_on_executor_thread(cb: Arc<dyn Fn() + Send + Sync + 'static>) {
    ClientAction::Call(cb).emit();
}

pub(crate) fn stop_executor_thread() {
    ClientAction::Stop.emit();
}
