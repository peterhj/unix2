extern crate libc;

use std::convert::{TryInto};
use std::io::{Error};
use std::mem::{MaybeUninit, zeroed};
use std::os::unix::io::{AsRawFd};
use std::time::{Duration};

pub fn set_gid(gid: u32) -> Result<(), Error> {
  unsafe {
    let res = libc::setgroups(1, &gid);
    if res != 0 {
      return Err(Error::last_os_error());
    }
    let res = libc::setgid(gid);
    if res != 0 {
      return Err(Error::last_os_error());
    }
  }
  Ok(())
}

pub fn set_uid(uid: u32) -> Result<(), Error> {
  unsafe {
    let res = libc::setuid(uid);
    if res != 0 {
      return Err(Error::last_os_error());
    }
  }
  Ok(())
}

pub fn umask(mode: u32) -> Result<u32, Error> {
  unsafe {
    let prev = libc::umask(mode);
    Ok(prev)
  }
}

#[derive(Clone, Copy)]
pub struct FdSet {
  raw:  libc::fd_set,
}

impl Default for FdSet {
  fn default() -> FdSet {
    FdSet::new()
  }
}

impl FdSet {
  pub fn new() -> FdSet {
    let mut raw = MaybeUninit::uninit();
    unsafe {
      libc::FD_ZERO(raw.as_mut_ptr());
      FdSet{raw: raw.assume_init()}
    }
  }

  pub fn insert<F: AsRawFd>(&mut self, fd: &F) {
    let fd = fd.as_raw_fd();
    unsafe {
      libc::FD_SET(fd, &mut self.raw);
    }
  }
}

pub fn select_read_fd_timeout<F: AsRawFd>(fd: &F, timeout: Duration) -> Result<Option<()>, Error> {
  let mut read = FdSet::new();
  let mut write = FdSet::new();
  let mut except = FdSet::new();
  read.insert(fd);
  let fd = fd.as_raw_fd();
  let ub = fd + 1;
  assert!(fd < ub);
  unsafe {
    let mut tval: libc::timeval = zeroed();
    tval.tv_sec = timeout.as_secs().try_into().unwrap();
    tval.tv_usec = timeout.subsec_micros().try_into().unwrap();
    let res = libc::select(ub, &mut read.raw, &mut write.raw, &mut except.raw, &mut tval);
    if res < 0 {
      return Err(Error::last_os_error());
    }
    if res == 0 {
      Ok(None)
    } else {
      Ok(Some(()))
    }
  }
}
