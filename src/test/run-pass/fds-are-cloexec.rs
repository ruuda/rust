// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// ignore-windows
// ignore-android

#![feature(libc)]

extern crate libc;

use std::env;
use std::fs::{self, File};
use std::io;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::os::unix::prelude::*;
use std::process::Command;
use std::thread;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        parent()
    } else {
        child(&args)
    }
}

fn parent() {
    let file = File::open("Makefile").unwrap();
    let _dir = fs::read_dir("/").unwrap();
    let tcp1 = TcpListener::bind("127.0.0.1:0").unwrap();
    assert_eq!(tcp1.as_raw_fd(), file.as_raw_fd() + 2);
    let tcp2 = tcp1.try_clone().unwrap();
    let addr = tcp1.local_addr().unwrap();
    let t = thread::scoped(|| TcpStream::connect(addr).unwrap());
    let tcp3 = tcp1.accept().unwrap().0;
    let tcp4 = t.join();
    let tcp5 = tcp3.try_clone().unwrap();
    let tcp6 = tcp4.try_clone().unwrap();
    let udp1 = UdpSocket::bind("127.0.0.1:0").unwrap();
    let udp2 = udp1.try_clone().unwrap();

    let status = Command::new(env::args().next().unwrap())
                        .arg(file.as_raw_fd().to_string())
                        .arg((file.as_raw_fd() + 1).to_string())
                        .arg(tcp1.as_raw_fd().to_string())
                        .arg(tcp2.as_raw_fd().to_string())
                        .arg(tcp3.as_raw_fd().to_string())
                        .arg(tcp4.as_raw_fd().to_string())
                        .arg(tcp5.as_raw_fd().to_string())
                        .arg(tcp6.as_raw_fd().to_string())
                        .arg(udp1.as_raw_fd().to_string())
                        .arg(udp2.as_raw_fd().to_string())
                        .status()
                        .unwrap();
    assert!(status.success());
}

fn child(args: &[String]) {
    let mut b = [0u8; 2];
    for arg in &args[1..] {
        let fd: libc::c_int = arg.parse().unwrap();
        unsafe {
            assert_eq!(libc::read(fd, b.as_mut_ptr() as *mut _, 2), -1);
            assert_eq!(io::Error::last_os_error().raw_os_error(),
                       Some(libc::EBADF));
        }
    }
}
