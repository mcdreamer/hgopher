use std::env;
use rusthgopher::server::ServerConfig;

fn main() {
    let args: Vec<String> = env::args().collect();
    let addr = &args[1];
    let port = &args[2];
    let root = &args[3];
    let cfg = ServerConfig::new(addr, port.parse().expect("Invalid port number"), root);
    rusthgopher::server::run(cfg);
}