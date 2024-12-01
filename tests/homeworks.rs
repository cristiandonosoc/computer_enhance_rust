mod common;

fn evaluate_debug_logging() {
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
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
    ];
    for listing in listings {
        if let Err(e) = common::simulation::run_simulation_test(listing) {
            assert!(false, "{}", e);
        }
    }
}
