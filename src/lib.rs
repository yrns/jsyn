//use fundsp::prelude::Net64;
use janetrs::{declare_janet_mod, janet_fn, jpanic, Janet, JanetAbstract};
use net::Net;

pub mod net;
pub mod play;

// Use https://github.com/Enselic/cargo-public-api at build time to read fundsp::hacker::*?

/// sine generator at hz.
#[janet_fn(arity(fix(1)))]
fn sine_hz(args: &mut [Janet]) -> Janet {
    let hz: f64 = args[0]
        .try_unwrap()
        .unwrap_or_else(|e| jpanic!("error: {}", e));
    Net::from(fundsp::hacker::sine_hz(hz)).into()
}

#[janet_fn(arity(fix(1)))]
fn play(args: &mut [Janet]) -> Janet {
    let net: JanetAbstract = args[0]
        .try_unwrap()
        .unwrap_or_else(|e| jpanic!("error: {}", e));
    let net: &Net = net.get().unwrap_or_else(|e| jpanic!("error: {}", e));

    // We can't move out of Janet so clone.
    let net = (*net).clone();
    JanetAbstract::new(play::play(net).unwrap()).into()
}

declare_janet_mod!("jsyn";
    {"sine-hz", sine_hz},
    {"play", play},
);
