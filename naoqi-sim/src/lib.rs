#![deny(unreachable_pub, unsafe_code)]
#![warn(
    missing_docs,
    clippy::all,
    clippy::clone_on_ref_ptr,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::implicit_clone,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::mixed_read_write_in_expression,
    clippy::multiple_inherent_impl,
    clippy::mutex_atomic,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::mod_module_files,
    clippy::str_to_string,
    clippy::string_slice,
    clippy::todo,
    clippy::try_err,
    clippy::unimplemented,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::use_debug
)]
// Deny warnings in doc test.
#![doc(test(attr(deny(warnings))))]
#![doc = include_str!("../README.md")]

pub mod alvalue;
pub mod body;
pub mod log;
pub mod memory;
pub mod robot;
pub mod script;
pub mod services;
pub mod simulator;

pub use self::{
    alvalue::AlValue,
    body::Body,
    log::{LogHub, LogMessage},
    memory::Memory,
    robot::RobotModel,
    script::Script,
    simulator::{Config, Simulator},
};

/// Builds a `qi` error carrying a message, the way NAOqi reports errors to callers.
pub(crate) fn error<M: std::fmt::Display>(message: M) -> qi::Error {
    qi::Error::Other(message.to_string().into())
}

/// Locks a mutex, recovering the guard if the lock was poisoned by a panicking task.
pub(crate) fn lock<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| {
        mutex.clear_poison();
        err.into_inner()
    })
}

/// The current wall clock time as NAOqi timestamps: seconds and microseconds since the epoch.
pub(crate) fn now_secs_usecs() -> (i32, i32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    (
        i32::try_from(now.as_secs()).unwrap_or(i32::MAX),
        i32::try_from(now.subsec_micros()).unwrap_or(0),
    )
}
