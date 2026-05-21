//! asdb — Phase 2/3: Stdio JSON-RPC server

mod query;
mod scan;
mod server;
mod storage;
mod transport;

use std::io::BufReader;
use transport::{read_lines, write_line};

fn main() {
    // tree-sitter grammars parse recursively. rayon's par_iter also uses the
    // *calling* thread as a worker, so both the main thread AND every rayon
    // worker need a generous stack.  Spawn everything on a 64 MB thread and
    // configure rayon workers to use 32 MB each.
    std::thread::Builder::new()
        .name("asdb-main".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .expect("spawn main thread")
        .join()
        .expect("main thread panicked");
}

fn run() {
    rayon::ThreadPoolBuilder::new()
        .stack_size(32 * 1024 * 1024)
        .build_global()
        .ok();

    let mut srv = server::Server::new();
    let stdin = BufReader::new(std::io::stdin());

    for line in read_lines(stdin) {
        for out in srv.handle_line(&line) {
            write_line(&out);
        }
    }
}
