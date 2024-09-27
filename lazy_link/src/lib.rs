#![no_std]

mod cache;
pub use cache::{
    Cache,
    NoCache,
    StaticAtomicCache,
    StaticCache,
};
pub use lazy_link_derive::lazy_link;
