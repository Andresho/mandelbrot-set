use std::sync::{Arc, Mutex};

pub struct SyncFlagSender {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagSender {
    pub fn set(&mut self, state: bool) -> Result<(), ()> {
        if let Ok(mut v) = self.inner.lock() {
            *v = state;
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Clone)]
pub struct SyncFlagReceiver {
    inner: Arc<Mutex<bool>>,
}

impl SyncFlagReceiver {
    pub fn get(&self) -> Result<bool, ()> {
        if let Ok(v) = self.inner.lock() {
            Ok(*v)
        } else {
            Err(())
        }
    }
}

pub fn new_syncflag(initial_state: bool) -> (SyncFlagSender, SyncFlagReceiver) {
    let state = Arc::new(Mutex::new(initial_state));
    let tx = SyncFlagSender { inner: state.clone() };
    let rx = SyncFlagReceiver { inner: state.clone() };

    return (tx, rx);
}
