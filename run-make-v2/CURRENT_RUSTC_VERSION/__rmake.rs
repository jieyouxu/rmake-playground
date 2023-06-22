extern crate rmake_support;

fn main() {
    let mut scx = rmake_support::RmakeSupportContext::init();

    println!("Hello World!");
    println!("scx.resolve_env_var() = {:?}", scx.resolve_env_var("HOST_RPATH_ENV"));
}
