//! asdb — Phase 2: Stdio JSON-RPC server

mod scan;
mod server;
mod storage;
mod transport;

use std::io::BufReader;
use transport::{read_lines, write_line};

fn main() {
    let mut srv = server::Server::new();
    let stdin = BufReader::new(std::io::stdin());

    for line in read_lines(stdin) {
        for out in srv.handle_line(&line) {
            write_line(&out);
        }
    }
}
