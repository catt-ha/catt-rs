// This is a somewhat hacky wrapper for std receivers for use in mioco.
// Most of the implementation details were taken from mioco::sync::Mutex's approach.

use std::sync::mpsc;
use mioco;

pub struct Receiver<T> {
    recv: mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn new(recv: mpsc::Receiver<T>) -> Receiver<T> {
        Receiver { recv: recv }
    }

    pub fn recv(&self) -> Result<T, mpsc::RecvError> {
        if mioco::in_coroutine() {
            self.recv_in_mioco()
        } else {
            self.recv.recv()
        }
    }

    pub fn recv_in_mioco(&self) -> Result<T, mpsc::RecvError> {
        loop {
            match self.recv.try_recv() {
                Ok(it) => return Ok(it),
                Err(err) => {
                    match err {
                        mpsc::TryRecvError::Empty => mioco::yield_now(),
                        mpsc::TryRecvError::Disconnected => return Err(mpsc::RecvError {}),
                    }
                }
            }
        }
    }

    pub fn try_recv(&self) -> Result<T, mpsc::TryRecvError> {
        self.recv.try_recv()
    }
}

pub struct IntoIter<T> {
    recv: Receiver<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.recv.recv() {
            Ok(it) => Some(it),
            Err(_) => None,
        }
    }
}

impl<T> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { recv: self }
    }
}
