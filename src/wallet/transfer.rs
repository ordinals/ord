pub(crate) mod rune_transfer;
pub(crate) mod inscription_transfer;
pub(crate) mod satpoint_transfer;
pub(crate) mod sat_transfer;
pub(crate) mod amount_tranfer;

use {super::*, crate::outgoing::Outgoing, base64::Engine, bitcoin::psbt::Psbt};
use crate::subcommand::wallet::send::Output;

pub(crate) trait Transfer{
    fn get_outgoing(&self) -> Outgoing;
    fn create_unsigned_transaction(&self,
                                   wallet: &Wallet,
                                   destination: Address,
                                   postage: Option<Amount>,
                                   fee_rate: FeeRate) -> Result<Transaction>;
    fn send(&self, wallet: &Wallet,
            dry_run: bool, destination: Address,
            postage: Option<Amount>,
            fee_rate: FeeRate) -> SubcommandResult {
        let unsigned_transaction = self.create_unsigned_transaction(wallet, destination, postage, fee_rate)?;

        let unspent_outputs = wallet.utxos();

        let (txid, psbt) = if dry_run {
            let psbt = wallet
                .bitcoin_client()
                .wallet_process_psbt(
                    &base64::engine::general_purpose::STANDARD
                        .encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
                    Some(false),
                    None,
                    None,
                )?
                .psbt;

            (unsigned_transaction.txid(), psbt)
        } else {
            let psbt = wallet
                .bitcoin_client()
                .wallet_process_psbt(
                    &base64::engine::general_purpose::STANDARD
                        .encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
                    Some(true),
                    None,
                    None,
                )?
                .psbt;

            let signed_tx = wallet
                .bitcoin_client()
                .finalize_psbt(&psbt, None)?
                .hex
                .ok_or_else(|| anyhow!("unable to sign transaction"))?;

            (
                wallet.bitcoin_client().send_raw_transaction(&signed_tx)?,
                psbt,
            )
        };

        let mut fee = 0;
        for txin in unsigned_transaction.input.iter() {
            let Some(txout) = unspent_outputs.get(&txin.previous_output) else {
                panic!("input {} not found in utxos", txin.previous_output);
            };
            fee += txout.value;
        }

        for txout in unsigned_transaction.output.iter() {
            fee = fee.checked_sub(txout.value).unwrap();
        }

        Ok(Some(Box::new(Output {
            txid,
            psbt,
            outgoing: self.get_outgoing(),
            fee,
        })))
    }

    fn create_unsigned_send_satpoint_transaction(
        &self,
        wallet: &Wallet,
        destination: Address,
        satpoint: SatPoint,
        postage: Option<Amount>,
        fee_rate: FeeRate,
    ) -> Result<Transaction> {
        let runic_outputs = wallet.get_runic_outputs()?;

        ensure!(
      !runic_outputs.contains(&satpoint.outpoint),
      "runic outpoints may not be sent by satpoint"
    );

        let change = [wallet.get_change_address()?, wallet.get_change_address()?];

        let postage = if let Some(postage) = postage {
            Target::ExactPostage(postage)
        } else {
            Target::Postage
        };

        Ok(
            TransactionBuilder::new(
                satpoint,
                wallet.inscriptions().clone(),
                wallet.utxos().clone(),
                wallet.locked_utxos().clone().into_keys().collect(),
                runic_outputs,
                destination.clone(),
                change,
                fee_rate,
                postage,
            )
                .build_transaction()?,
        )
    }

}