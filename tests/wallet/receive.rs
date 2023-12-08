use {super::*, ord::subcommand::wallet::receive::{Output, PsbtOutput}}; // Import PsbtOutput for the updated test

#[test]
fn receive() {
    let rpc_server = test_bitcoincore_rpc::spawn();
    create_wallet(&rpc_server);

    // Updated command to request a PSBT instead of a regular transaction
    let output = CommandBuilder::new("wallet receive --psbt")
        .rpc_server(&rpc_server)
        .run_and_deserialize_output::<PsbtOutput>(); // Use PsbtOutput for the updated test

    // Assert that a valid PSBT is returned
    assert!(output.psbt.is_valid());
}
