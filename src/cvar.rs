use std::sync::{Mutex, Condvar, MutexGuard};
use std::sync::Arc;

#[derive(Clone)]
pub struct CVar(Arc<(Mutex<bool>, Condvar)>);

impl CVar {
    pub fn new() -> Self {
        CVar(Arc::new((Mutex::new(false), Condvar::new())))
    }

    pub fn wait(&self) -> MutexGuard<bool> {
        let &(ref pred, ref c_var) = &*self.0;
        let mut pred = match pred.lock() {
            Ok(l) => l,
            Err(e) => e.into_inner(),
        };
        while !*pred {
            pred = match c_var.wait(pred) {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            }
        }
        pred
    }

    pub fn notify_one(&self) {
        let &(ref pred, ref c_var) = &*self.0;
        let mut pred = match pred.lock() {
            Ok(l) => l,
            Err(e) => e.into_inner(),
        };
        *pred = true;
        c_var.notify_one();
    }

    pub fn notify_all(&self) {
        let &(ref pred, ref c_var) = &*self.0;
        let mut pred = match pred.lock() {
            Ok(l) => l,
            Err(e) => e.into_inner(),
        };
        *pred = true;
        c_var.notify_all();
    }

    pub fn reset(&self) {
        let &(ref pred, _) = &*self.0;
        let mut pred = match pred.lock() {
            Ok(l) => l,
            Err(e) => e.into_inner(),
        };
        *pred = false;
    }
}
