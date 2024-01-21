/* Derived from the epoll crate by Nathan Sizemore:

Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
If a copy of the MPL was not distributed with this file,
You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::{self, Error};
use std::mem::{zeroed};
use std::ops::{BitAnd, BitOr};
use std::os::unix::io::{AsRawFd, RawFd};

#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum Control {
    /// Indicates an addition to the interest list.
    EPOLL_CTL_ADD = libc::EPOLL_CTL_ADD,
    /// Indicates a modification of flags for an interest already in list.
    EPOLL_CTL_MOD = libc::EPOLL_CTL_MOD,
    /// Indicates a removal of an interest from the list.
    EPOLL_CTL_DEL = libc::EPOLL_CTL_DEL,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct Events {
    pub bits: u32,
}

impl Events {
    #[inline]
    pub fn empty() -> Events {
        Events{bits: 0}
    }

    #[inline]
    pub fn from_bits(bits: u32) -> Events {
        Events{bits}
    }

    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits
    }
}

impl BitAnd for Events {
    type Output = Events;

    #[inline]
    fn bitand(self, rhs: Events) -> Events {
        Events{bits: self.bits & rhs.bits}
    }
}

impl BitOr for Events {
    type Output = Events;

    #[inline]
    fn bitor(self, rhs: Events) -> Events {
        Events{bits: self.bits | rhs.bits}
    }
}

/// Sets the Edge Triggered behavior for the associated file descriptor.
///
/// The default behavior for epoll is Level Triggered.
pub const EPOLLET: Events = Events{bits: libc::EPOLLET as u32};

/// The associated file is available for read operations.
pub const EPOLLIN: Events = Events{bits: libc::EPOLLIN as u32};

/// Error condition happened on the associated file descriptor.
///
/// `wait` will always wait for this event; is not necessary to set it in events.
pub const EPOLLERR: Events = Events{bits: libc::EPOLLERR as u32};

/// Hang up happened on the associated file descriptor.
///
/// `wait` will always wait for this event; it is not necessary to set it in events.
/// Note that when reading from a channel such as a pipe or a stream socket, this event
/// merely indicates that the peer closed its end of the channel. Subsequent reads from
/// the channel will return 0 (end of file) only after all outstanding data in the
/// channel has been consumed.
pub const EPOLLHUP: Events = Events{bits: libc::EPOLLHUP as u32};

/// The associated file is available for write operations.
pub const EPOLLOUT: Events = Events{bits: libc::EPOLLOUT as u32};

/// There is urgent data available for read operations.
pub const EPOLLPRI: Events = Events{bits: libc::EPOLLPRI as u32};

/// Stream socket peer closed connection, or shut down writing half of connection.
///
/// This flag is especially useful for writing simple code to detect peer shutdown when
/// using Edge Triggered monitoring.
pub const EPOLLRDHUP: Events = Events{bits: libc::EPOLLRDHUP as u32};

/// If `EPOLLONESHOT` and `EPOLLET` are clear and the process has the `CAP_BLOCK_SUSPEND`
/// capability, ensure that the system does not enter "suspend" or "hibernate" while this
/// event is pending or being processed.
///
/// The event is considered as being "processed" from the time when it is returned by
/// a call to `wait` until the next call to `wait` on the same `EpollInstance`
/// descriptor, the closure of that file descriptor, the removal of the event file
/// descriptor with `EPOLL_CTL_DEL`, or the clearing of `EPOLLWAKEUP` for the event file
/// descriptor with `EPOLL_CTL_MOD`.
pub const EPOLLWAKEUP: Events = Events{bits: libc::EPOLLWAKEUP as u32};

/// Sets the one-shot behavior for the associated file descriptor.
///
/// This means that after an event is pulled out with `wait` the associated file
/// descriptor is internally disabled and no other events will be reported by the epoll
/// interface.  The user must call `ctl` with `EPOLL_CTL_MOD` to rearm the file
/// descriptor with a new event mask.
pub const EPOLLONESHOT: Events = Events{bits: libc::EPOLLONESHOT as u32};

/// Sets an exclusive wakeup mode for the epoll file descriptor that is being attached to
/// the target file descriptor, `fd`. When a wakeup event occurs and multiple epoll file
/// descriptors are attached to the same target file using `EPOLLEXCLUSIVE`, one or more of
/// the epoll file descriptors will receive an event with `wait`. The default in this
/// scenario (when `EPOLLEXCLUSIVE` is not set) is for all epoll file descriptors to
/// receive an event. `EPOLLEXCLUSIVE` is thus useful for avoiding thundering herd problems
/// in certain scenarios.
///
/// If the same file descriptor is in multiple epoll instances, some with the
/// `EPOLLEXCLUSIVE` flag, and others without, then events will be provided to all epoll
/// instances that did not specify `EPOLLEXCLUSIVE`, and at least one of the epoll
/// instances that did specify `EPOLLEXCLUSIVE`.
///
/// The following values may be specified in conjunction with `EPOLLEXCLUSIVE`: `EPOLLIN`,
/// `EPOLLOUT`, `EPOLLWAKEUP`, and `EPOLLET`. `EPOLLHUP` and `EPOLLERR` can also be
/// specified, but this is not required: as usual, these events are always reported if they
/// occur, regardless of whether they are specified in `Events`. Attempts to specify other
/// values in `Events` yield the error `EINVAL`.
///
/// `EPOLLEXCLUSIVE` may be used only in an `EPOLL_CTL_ADD` operation; attempts to employ
/// it with `EPOLL_CTL_MOD` yield an error. If `EPOLLEXCLUSIVE` has been set using `ctl`,
/// then a subsequent `EPOLL_CTL_MOD` on the same `epfd`, `fd` pair yields an error. A call
/// to `ctl` that specifies `EPOLLEXCLUSIVE` in `Events` and specifies the target file
/// descriptor `fd` as an epoll instance will likewise fail. The error in all of these
/// cases is `EINVAL`.
///
/// The `EPOLLEXCLUSIVE` flag is an input flag for the `Event.events` field when calling
/// `ctl`; it is never returned by `wait`.
pub const EPOLLEXCLUSIVE: Events = Events{bits: libc::EPOLLEXCLUSIVE as u32};

/// 'libc::epoll_event' equivalent.
///
/// SAFETY: This must have the same definition and repr(packed)
/// as `libc::epoll_event`.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    any(all(target_arch = "x86",
            not(target_env = "musl"),
            not(target_os = "android")),
        target_arch = "x86_64"),
    repr(packed))]
pub struct Event {
    events: u32,
    data: u64,
}

impl Default for Event {
    #[inline]
    fn default() -> Event {
        Event::new(Events::empty(), 0)
    }
}

impl Event {
    #[inline]
    pub fn new(events: Events, data: u64) -> Event {
        let mut ev: Event = unsafe { zeroed() };
        ev.events = events.bits();
        ev.data = data;
        ev
    }

    #[inline]
    pub fn events(&self) -> Events {
        Events::from_bits(self.events)
    }

    #[inline]
    pub fn raw_events(&self) -> u32 {
        self.events
    }

    #[inline]
    pub fn raw_data(&self) -> u64 {
        self.data
    }
}

fn cvt(result: libc::c_int) -> io::Result<libc::c_int> {
    if result < 0 {
        Err(Error::last_os_error())
    } else {
        Ok(result)
    }
}

pub struct Epoll {
    epfd: RawFd,
}

impl Drop for Epoll {
    fn drop(&mut self) {
        let epfd = self.epfd;
        let _ = cvt(unsafe { libc::close(epfd) });
        self.epfd = -1;
    }
}

impl Epoll {
    /// Creates a new epoll file descriptor.
    ///
    /// If `cloexec` is true, `FD_CLOEXEC` will be set on the returned file descriptor.
    ///
    /// ## Notes
    ///
    /// * `epoll_create1()` is the underlying syscall.
    pub fn create(cloexec: bool) -> io::Result<Epoll> {
        let flags = if cloexec { libc::EPOLL_CLOEXEC } else { 0 };
        let epfd = cvt(unsafe { libc::epoll_create1(flags) })?;
        Ok(Epoll{epfd})
    }

    /// Safe wrapper for `libc::epoll_ctl`
    pub fn ctl(&self, op: Control, fd: RawFd, mut event: Event) -> io::Result<()> {
        let epfd = self.epfd;
        let e = &mut event as *mut _ as *mut libc::epoll_event;
        cvt(unsafe { libc::epoll_ctl(epfd, op as i32, fd, e) })?;
        Ok(())
    }

    /// Safe wrapper for `libc::epoll_wait`
    ///
    /// ## Notes
    ///
    /// * If `timeout` is negative, it will block until an event is received.
    pub fn wait(&self, timeout: i32, buf: &mut [Event]) -> io::Result<usize> {
        let epfd = self.epfd;
        let timeout = if timeout < -1 { -1 } else { timeout };
        let num_events = cvt(unsafe {
            libc::epoll_wait(
                epfd,
                buf.as_mut_ptr() as *mut libc::epoll_event,
                buf.len() as i32,
                timeout,
            )
        })? as usize;
        Ok(num_events)
    }
}

impl AsRawFd for Epoll {
    fn as_raw_fd(&self) -> RawFd {
        self.epfd
    }
}
