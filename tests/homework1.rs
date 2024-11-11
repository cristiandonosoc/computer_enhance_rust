#![allow(dead_code)]

mod common;

#[test]
fn test_listings() {
    if let Err(e) = common::run_nasm_test("listing_37.asm") {
        assert!(false, "{}", e);
    }
    //assert!(common::run_nasm_test("listing_38.asm").is_ok());
}
