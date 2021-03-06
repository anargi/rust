// Copyright 2018 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(unused_mut)]
// ignore-wasm32
// ignore-emscripten

// compile-flags: -C debug_assertions=yes

#![feature(const_fn, libc)]
#![allow(const_err)]

extern crate libc;

use std::env;
use std::process::{Command, Stdio};

// this will panic in debug mode and overflow in release mode
const fn bar() -> usize { 0 - 1 }

fn foo() {
    let _: &'static _ = &bar();
}

#[cfg(unix)]
fn check_status(status: std::process::ExitStatus)
{
    use libc;
    use std::os::unix::process::ExitStatusExt;

    assert!(status.signal() == Some(libc::SIGILL)
            || status.signal() == Some(libc::SIGABRT));
}

#[cfg(not(unix))]
fn check_status(status: std::process::ExitStatus)
{
    assert!(!status.success());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "test" {
        foo();
        return;
    }

    let mut p = Command::new(&args[0])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .arg("test").output().unwrap();
    check_status(p.status);
}
