//! Resource handles reachable from [`Skailar`](crate::Skailar).
//!
//! Each handle borrows the client and exposes the methods for one API area.
//! They are obtained from the client's accessor methods (`client.chat()`,
//! `client.models()`, …) rather than constructed directly.

pub mod audio;
pub mod chat;
pub mod images;
pub mod models;
pub mod uploads;
