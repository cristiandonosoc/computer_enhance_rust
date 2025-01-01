pub struct DeferStruct<F>
where F: FnMut() {
    pub f: F,
}


impl<F: FnMut()> Drop for DeferStruct<F> {
    fn drop(&mut self) {
        (self.f)();
    }
}


#[macro_export]
macro_rules! defer {
    ($e:expr) => {
        let _defer = $crate::utils::DeferStruct { f: || -> () { $e; }};
    }
}
