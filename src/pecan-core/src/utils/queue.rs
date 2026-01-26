use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct Queue<T> {
    inner: Mutex<Inner<T>>,
    not_empty: Condvar,
    not_full: Condvar,
}

struct Inner<T> {
    buf: VecDeque<T>,
    capacity: usize,
    closed: bool,
}

impl<T> Queue<T> {
    pub fn bounded(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            inner: Mutex::new(Inner {
                buf: VecDeque::with_capacity(capacity),
                capacity,
                closed: false,
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(_) => return Err(value),
        };
        while inner.buf.len() == inner.capacity && !inner.closed {
            inner = match self.not_full.wait(inner) {
                Ok(inner) => inner,
                Err(_) => return Err(value),
            };
        }
        if inner.closed {
            return Err(value);
        }
        inner.buf.push_back(value);
        self.not_empty.notify_one();
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(_) => return None,
        };
        while inner.buf.is_empty() && !inner.closed {
            inner = match self.not_empty.wait(inner) {
                Ok(inner) => inner,
                Err(_) => return None,
            };
        }
        if let Some(v) = inner.buf.pop_front() {
            self.not_full.notify_one();
            Some(v)
        } else {
            None
        }
    }

    pub fn try_push(&self, value: T) -> Result<(), TryPushError<T>> {
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(_) => return Err(TryPushError::Poisoned(value)),
        };
        if inner.closed {
            return Err(TryPushError::Closed(value));
        }
        if inner.buf.len() == inner.capacity {
            return Err(TryPushError::Full(value));
        }
        inner.buf.push_back(value);
        self.not_empty.notify_one();
        Ok(())
    }

    pub fn try_pop(&self) -> Result<T, TryPopError> {
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(_) => return Err(TryPopError::Poisoned),
        };
        if let Some(v) = inner.buf.pop_front() {
            self.not_full.notify_one();
            Ok(v)
        } else if inner.closed {
            Err(TryPopError::Closed)
        } else {
            Err(TryPopError::Empty)
        }
    }

    pub fn close(&self) {
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(_) => return,
        };
        inner.closed = true;
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }

    pub fn is_closed(&self) -> bool {
        match self.inner.lock() {
            Ok(inner) => inner.closed,
            Err(_) => true,
        }
    }

    pub fn len(&self) -> usize {
        match self.inner.lock() {
            Ok(inner) => inner.buf.len(),
            Err(_) => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub enum TryPushError<T> {
    Full(T),
    Closed(T),
    Poisoned(T),
}

pub enum TryPopError {
    Empty,
    Closed,
    Poisoned,
}

#[cfg(test)]
mod tests {
    use super::{Queue, TryPopError, TryPushError};

    #[test]
    fn try_push_and_try_pop_roundtrip() {
        let queue = Queue::bounded(2);

        assert!(matches!(queue.try_pop(), Err(TryPopError::Empty)));
        assert!(queue.try_push(10).is_ok());
        assert!(queue.try_push(20).is_ok());

        assert_eq!(queue.len(), 2);
        assert!(matches!(queue.try_push(30), Err(TryPushError::Full(30))));
        assert!(matches!(queue.try_pop(), Ok(10)));
        assert!(matches!(queue.try_pop(), Ok(20)));
        assert!(matches!(queue.try_pop(), Err(TryPopError::Empty)));
    }

    #[test]
    fn close_prevents_future_pushes() {
        let queue = Queue::bounded(1);

        queue.close();

        assert!(queue.is_closed());
        assert!(matches!(queue.try_push(1), Err(TryPushError::Closed(1))));
        assert!(matches!(queue.push(2), Err(2)));
        assert!(matches!(queue.try_pop(), Err(TryPopError::Closed)));
    }
}
