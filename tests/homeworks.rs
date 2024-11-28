mod common;

#[test]
fn homework1() {
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

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
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let listings = ["listing_39.asm"];

    for listing in listings {
        if let Err(e) = common::run_nasm_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework3() {
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let listings = ["listing_41.asm"];

    for listing in listings {
        if let Err(e) = common::run_nasm_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework4() {
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

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
