use std::time::{Duration, Instant};
use std::fmt;
use lifx_core::Message;

type BoxUpdateFn<T> = Box<dyn FnMut(Option<&T>) -> Option<Message> + Send>;

pub(crate) struct RefreshableData<T> where T: Clone {
    data: Option<T>,
    max_age: Option<Duration>,
    last_updated: Instant,
    refresh: BoxUpdateFn<T>,
}

impl<T> RefreshableData<T> where T: Clone {
    pub(crate) fn with_config(max_age: Duration, message: Message) -> Self
    {
        Self::with_dyn_config(max_age, move |_| Some(message.clone()))
    }

    pub(crate) fn with_dyn_config<F>(max_age: Duration, refresh: F) -> Self
    where
        F: FnMut(Option<&T>) -> Option<Message> + Send + 'static,
    {
        Self {
            data: None,
            max_age: Some(max_age),
            last_updated: Instant::now(),
            refresh: Box::new(refresh),
        }
    }

    pub(crate) fn needs_update(&self) -> bool {
        self.max_age
            .filter(|&max_age| self.data.is_none() || self.last_updated.elapsed() > max_age)
            .is_some()
    }

    pub(crate) fn check(&mut self) -> Option<Message> {
        if self.needs_update() {
            (self.refresh)(self.data.as_ref())
        } else {
            None
        }
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

    pub fn keep_updated(&mut self, max_age: Option<Duration>) {
        self.max_age = max_age;
    }

    pub fn get(&mut self) -> Option<&T> {
        // self.check()
        self.as_ref()
    }

    pub fn get_blocking(&self) -> T {
        self.as_ref().cloned().unwrap()
    }
}

impl<T: fmt::Debug> fmt::Debug for RefreshableData<T> where T: Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RefreshableData {{ data: {:?}, last_updated: {:?} }}", self.data, self.last_updated)
    }
}

