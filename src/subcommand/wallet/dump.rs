use super::*;
use std::collections::BTreeMap;

pub(crate) fn run(wallet: Wallet, matches: &ArgMatches) -> SubcommandResult {
    let dump_private_keys = matches.is_present("dump-private-keys");

    let warning_message = if dump_private_keys {
        "==========================================\n\
         = THIS STRING CONTAINS YOUR PRIVATE KEYS =\n\
         =        DO NOT SHARE WITH ANYONE        =\n\
         =========================================="
    } else {
        "==========================================\n\
         =        PUBLIC DESCRIPTOR OUTPUT        =\n\
         =       HANDLE WITH CARE, BUT LESS       =\n\
         =       SENSITIVE THAN PRIVATE KEYS      =\n\
         =========================================="
    };

    eprintln!(warning_message);

    Ok(Some(Box::new(
        wallet.bitcoin_client().list_descriptors(Some(dump_private_keys))?,
    )))
}
