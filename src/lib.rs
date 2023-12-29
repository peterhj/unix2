extern crate libc;

use std::io::{Error};

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
