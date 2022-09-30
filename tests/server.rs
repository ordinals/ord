use super::*;

#[test]
fn run() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let builder = CommandBuilder::new("server")
    .rpc_server(&rpc_server)
    .expected_exit_signal(Signal::SIGINT);

  let mut command = builder.command();

  let mut child = command.spawn().unwrap();

  nix::sys::signal::kill(
    Pid::from_raw(child.id().try_into().unwrap()),
    Signal::SIGINT,
  )
  .unwrap();

  child.kill().unwrap();

  builder.check(child.wait_with_output().unwrap());
}
