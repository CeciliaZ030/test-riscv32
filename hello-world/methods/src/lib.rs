include!(concat!(env!("OUT_DIR"), "/methods.rs"));
// include!(concat!(env!("OUT_DIR"), "/test.rs"));
include!("./methods.rs");
pub mod guest_bin;
pub mod guest_test;
pub mod methods;
