use core::server::ApiServer;
use std::thread;

mod core;

fn main() {
  env_logger::init();
  let ports = vec![8000, 8001, 8002];
  let mut handles = vec![];

  for port in ports {
    let handle = thread::spawn(move || {
      let runtime = tokio::runtime::Runtime::new().unwrap();
      let server = ApiServer::new(port);

      runtime.block_on(server.start());
    });

    handles.push(handle);
  }

  for handle in handles {
    handle.join().unwrap();
  }
}
