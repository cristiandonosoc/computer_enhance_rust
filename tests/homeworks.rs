#![allow(dead_code)]

mod common;

#[test]
fn homework1() {
    let listings = [
        "listing_37.asm",
        "listing_38.asm",
    ];

    for listing in listings {
        if let Err(e) = common::run_nasm_test(listing) {
            assert!(false, "{}", e);
        }
    }
}

#[test]
fn homework2() {
    let listings = [
        "listing_39.asm",
    ];

    for listing in listings {
        if let Err(e) = common::run_nasm_test(listing) {
            assert!(false, "{}", e);
        }
    }
}
