use std::sync::atomic::{AtomicU16, Ordering};

mod anvil;
mod api;
mod fork;
mod ganache;
mod pubsub;
mod transaction;
pub mod utils;
mod wsapi;

// keeps track of ports that can be used
pub static NEXT_PORT: AtomicU16 = AtomicU16::new(8546);

pub fn next_port() -> u16 {
    NEXT_PORT.fetch_add(1, Ordering::SeqCst)
}

#[allow(unused)]
pub(crate) fn init_tracing() {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

fn main() {}
