use std::sync::{Mutex, Condvar, MutexGuard};
use std::sync::Arc;

pub fn always_lock<G>(res: ::std::sync::LockResult<G>) -> G {
    match res {
        Ok(g) => g,
        Err(e) => {
            warn!("mutex poisoned: {}", e);
            e.into_inner()
        }
    }
}

#[derive(Clone)]
pub struct CVar(Arc<(Mutex<bool>, Condvar)>);

impl CVar {
    pub fn new() -> Self {
        CVar(Arc::new((Mutex::new(false), Condvar::new())))
    }

    pub fn wait(&self) -> MutexGuard<bool> {
        let &(ref pred, ref c_var) = &*self.0;
        let mut pred = always_lock(pred.lock());
        while !*pred {
            pred = always_lock(c_var.wait(pred));
        }
        pred
    }

    pub fn notify_one(&self) {
        let &(ref pred, ref c_var) = &*self.0;
        let mut pred = always_lock(pred.lock());
        *pred = true;
        c_var.notify_one();
    }

    pub fn notify_all(&self) {
        let &(ref pred, ref c_var) = &*self.0;
        let mut pred = always_lock(pred.lock());
        *pred = true;
        c_var.notify_all();
    }

    pub fn reset(&self) {
        let &(ref pred, _) = &*self.0;
        let mut pred = always_lock(pred.lock());
        *pred = false;
    }
}
