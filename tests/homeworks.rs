mod common;

use std::sync::atomic::{AtomicBool, Ordering};

fn evaluate_debug_logging() {
    static ACTIVATED: AtomicBool = AtomicBool::new(false);
    if !ACTIVATED.swap(true, Ordering::Relaxed) {
        // let filter = "info";
        // let filter = "debug";
        // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();
    }
}

#[test]
fn homework1() {
    evaluate_debug_logging();

    #[rustfmt::skip]
    let listings = [
        "listing_37.asm",
        "listing_38.asm"
    ];

    for listing in listings {
        if let Err(e) = common::run_nasm_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework2() {
    evaluate_debug_logging();

    #[rustfmt::skip]
    let listings = [
        "listing_39.asm",
    ];

    for listing in listings {
        if let Err(e) = common::run_nasm_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework3() {
    evaluate_debug_logging();

    #[rustfmt::skip]
    let listings = [
        "listing_41.asm",
    ];

    for listing in listings {
        if let Err(e) = common::run_nasm_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework4() {
    evaluate_debug_logging();

    #[rustfmt::skip]
    let listings = [
        "listing_43.asm",
        "listing_44.asm",
    ];
    for listing in listings {
        if let Err(e) = common::simulation::run_simulation_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework5() {
    evaluate_debug_logging();

    #[rustfmt::skip]
    let listings = [
        "listing_46.asm",
    ];
    for listing in listings {
        if let Err(e) = common::simulation::run_simulation_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework6() {
    evaluate_debug_logging();

    #[rustfmt::skip]
    let listings = [
        "listing_48.asm",
        "listing_49.asm",
    ];
    for listing in listings {
        if let Err(e) = common::simulation::run_simulation_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework7() {
    evaluate_debug_logging();

    #[rustfmt::skip]
    let listings = [
        "listing_51.asm",
    ];
    for listing in listings {
        if let Err(e) = common::simulation::run_simulation_test(listing) {
            assert!(false, "{}", e);
        }
    }
}
