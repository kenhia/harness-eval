//! feedgen: a tiny static-file HTTP server and fixture generator used to
//! develop and test feedhub without touching the real internet.
//!
//! The [`server`] module serves a directory over HTTP/1.1 with a
//! content-derived `ETag`, a `Last-Modified` header, and conditional-GET
//! (`If-None-Match` / `If-Modified-Since` -> `304`) support. The [`fixtures`]
//! module writes a documented corpus of feeds.

pub mod fixtures;
pub mod server;

pub use server::{serve_forever, spawn, Server};
