use {super::*, crate::outgoing::Outgoing, bitcoin::{opcodes}};

#[derive(Debug, Parser)]
pub struct Burn {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Target <AMOUNT> postage with sent inscriptions. [default: 10000 sat]"
  )]
  pub(crate) postage: Option<Amount>,
  outgoing: Outgoing,
}

impl Burn {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let burn_script = script::Builder::new().push_opcode(opcodes::all::OP_RETURN).into_script();

    let send_command = send::Send {
      dry_run: self.dry_run,
      fee_rate: self.fee_rate,
      postage: self.postage,
      address: send::ParsedAddress::ScriptBuf(burn_script),
      outgoing: self.outgoing,
    };

    send_command.run(wallet)
  }
}
