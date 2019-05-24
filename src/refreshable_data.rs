use failure::Error;
use std::time::{Duration, Instant};

type BoxUpdateFn<T, O> = Box<dyn FnMut(O, Option<&T>) -> Result<(), Error>>;

pub(crate) struct RefreshableData<T, O> {
    data: Option<T>,
    max_age: Duration,
    last_updated: Instant,
    refresh: BoxUpdateFn<T, O>,
}

impl<T, O> RefreshableData<T, O> {
    pub(crate) fn with_config<F>(max_age: Duration, refresh: F) -> Self
    where
        F: FnMut(O, Option<&T>) -> Result<(), Error> + 'static,
    {
        Self {
            data: None,
            max_age,
            last_updated: Instant::now(),
            refresh: Box::new(refresh),
        }
    }

    pub(crate) fn check(&mut self, opts: O) -> Result<(), Error> {
        if self.data.is_none() || self.last_updated.elapsed() > self.max_age {
            (self.refresh)(opts, self.data.as_ref())?;
        }

        Ok(())
    }

    pub(crate) fn update(&mut self, data: T) {
        self.data = Some(data);
    }

    pub(crate) fn as_ref(&self) -> Option<&T> {
        self.data.as_ref()
    }
}
