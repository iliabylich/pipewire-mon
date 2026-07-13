use crate::pipewire::Pipewire;
use anyhow::Result;
use pipewire_mon::Event;
use pollsendd::Service;
use rustix::event::{PollFd, PollFlags};
use std::os::fd::AsRawFd;

pub struct PwModService {
    pw: Pipewire,
    volume: Attribute<u8>,
    muted: Attribute<bool>,
}

impl PwModService {
    pub(crate) fn new() -> Result<Self> {
        let pw = Pipewire::connect()?;
        Ok(Self {
            pw,
            volume: Attribute::empty(),
            muted: Attribute::empty(),
        })
    }
}

impl Service for PwModService {
    type Error = PwMonServiceError;

    fn pollfds(&mut self) -> Result<impl Iterator<Item = rustix::event::PollFd<'_>>, Self::Error> {
        Ok([PollFd::new(&self.pw, PollFlags::IN)].into_iter())
    }

    fn owns_fd(&self, fd: i32) -> bool {
        self.pw.as_raw_fd() == fd
    }

    fn on_readable(
        &mut self,
        _fd: i32,
        mut broadcast: impl FnMut(&[u8]),
    ) -> Result<(), Self::Error> {
        let events = self.pw.drain()?;
        let mut volume_changed = false;
        let mut muted_changed = false;
        for event in events {
            match event {
                Event::Volume(volume) => volume_changed |= self.volume.apply(volume),
                Event::Mute(muted) => muted_changed |= self.muted.apply(muted),
            }
        }
        if volume_changed && let Some(volume) = self.volume.as_option() {
            let buf = Event::Volume(volume).serialize();
            (broadcast)(&buf);
        }
        if muted_changed && let Some(muted) = self.muted.as_option() {
            let buf = Event::Mute(muted).serialize();
            (broadcast)(&buf);
        }
        Ok(())
    }

    fn on_writable(&mut self, _fd: i32, _broadcast: impl FnMut(&[u8])) -> Result<(), Self::Error> {
        Ok(())
    }

    fn on_poll_error(
        &mut self,
        _fd: i32,
        revents: rustix::event::PollFlags,
    ) -> Result<(), Self::Error> {
        Err(anyhow::anyhow!("Pipewire poll error: got revents {revents:?}").into())
    }

    fn state_as_serialized_bufs(&self) -> impl Iterator<Item = Vec<u8>> {
        let mut bufs = vec![];
        if let Some(volume) = self.volume.as_option() {
            bufs.push(Event::Volume(volume).serialize().to_vec());
        }
        if let Some(muted) = self.muted.as_option() {
            bufs.push(Event::Mute(muted).serialize().to_vec());
        }
        bufs.into_iter()
    }
}

pub struct PwMonServiceError(anyhow::Error);
impl std::fmt::Display for PwMonServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Debug for PwMonServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl core::error::Error for PwMonServiceError {}
impl From<anyhow::Error> for PwMonServiceError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

struct Attribute<T>(Option<T>)
where
    T: Copy + PartialEq;
impl<T> Attribute<T>
where
    T: Copy + PartialEq,
{
    fn empty() -> Self {
        Self(None)
    }
    fn apply(&mut self, new: T) -> bool {
        match &mut self.0 {
            Some(current) if *current == new => false,
            Some(current) => {
                *current = new;
                true
            }
            None => {
                self.0 = Some(new);
                true
            }
        }
    }
    fn as_option(&self) -> Option<T> {
        self.0
    }
}
impl<T> std::ops::Deref for Attribute<T>
where
    T: Copy + PartialEq,
{
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
