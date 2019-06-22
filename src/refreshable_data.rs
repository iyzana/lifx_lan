use failure::Error;
use std::time::{Duration, Instant};
use std::fmt;

type BoxUpdateFn<T, O> = Box<dyn FnMut(&O, Option<&T>) -> Result<(), Error> + Send>;

pub(crate) struct RefreshableData<T, O> {
    data: Option<T>,
    max_age: Duration,
    last_updated: Instant,
    refresh: BoxUpdateFn<T, O>,
}

impl<T, O> RefreshableData<T, O> {
    pub(crate) fn with_config<F>(max_age: Duration, refresh: F) -> Self
    where
        F: FnMut(&O, Option<&T>) -> Result<(), Error> + Send + 'static,
    {
        Self {
            data: None,
            max_age,
            last_updated: Instant::now(),
            refresh: Box::new(refresh),
        }
    }

    pub(crate) fn check(&mut self, opts: &O) -> Result<(), Error> {
        if self.data.is_none() || self.last_updated.elapsed() > self.max_age {
            (self.refresh)(opts, self.data.as_ref())?;
            self.last_updated = Instant::now();
        }

        Ok(())
    }

    pub(crate) fn update(&mut self, data: T) {
        self.data = Some(data);
        self.last_updated = Instant::now();
    }

    pub(crate) fn as_ref(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub(crate) fn as_mut(&mut self) -> Option<&mut T> {
        self.data.as_mut()
    }
}

impl<T: fmt::Debug, O> fmt::Debug for RefreshableData<T, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RefreshableData {{ data: {:?}, last_updated: {:?} }}", self.data, self.last_updated)
    }
}

