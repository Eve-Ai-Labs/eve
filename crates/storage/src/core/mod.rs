use bincode::{
    config::{BigEndian, WithOtherEndian},
    DefaultOptions, Options,
};
use std::sync::LazyLock;

pub mod db;
pub mod error;
pub mod iter;
pub mod table;
pub mod tx;

pub static KEY_OPTIONS: LazyLock<WithOtherEndian<DefaultOptions, BigEndian>> =
    LazyLock::new(|| DefaultOptions::new().with_big_endian());

pub static VALUE_OPTIONS: LazyLock<DefaultOptions> = LazyLock::new(DefaultOptions::new);
