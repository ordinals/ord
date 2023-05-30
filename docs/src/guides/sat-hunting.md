Sat Hunting
===========

*This guide is out of date. Since it was written, the `ord` binary was changed
to only build the full satoshi index when the `--index-sats` flag is supplied.
Additionally, `ord` now has a built-in wallet that wraps a Bitcoin Core wallet.
See `ord wallet --help`.*

Ordinal hunting is difficult but rewarding. The feeling of owning a wallet full
of UTXOs, redolent with the scent of rare and exotic sats, is beyond compare.

Ordinals are numbers for satoshis. Every satoshi has an ordinal number and
every ordinal number has a satoshi.

Preparation
-----------

There are a few things you'll need before you start.

1. First, you'll need a synced Bitcoin Core node with a transaction index. To
   turn on transaction indexing, pass `-txindex` on the command-line:

   ```sh
   bitcoind -txindex
   ```

   Or put the following in your [Bitcoin configuration
   file](https://github.com/bitcoin/bitcoin/blob/master/doc/bitcoin-conf.md#configuration-file-path):

   ```
   txindex=1
   ```

   Launch it and wait for it to catch up to the chain tip, at which point the
   following command should print out the current block height:

   ```sh
   bitcoin-cli getblockcount
   ```

2. Second, you'll need a synced `ord` index.

   - Get a copy of `ord` from [the repo](https://github.com/ordinals/ord/).

   - Run `RUST_LOG=info ord index`. It should connect to your bitcoin core
     node and start indexing.

   - Wait for it to finish indexing.

3. Third, you'll need a wallet with UTXOs that you want to search.

Searching for Rare Ordinals
---------------------------

### Searching for Rare Ordinals in a Bitcoin Core Wallet

The `ord wallet` command is just a wrapper around Bitcoin Core's RPC API, so
searching for rare ordinals in a Bitcoin Core wallet is Easy. Assuming your
wallet is named `foo`:

1. Load your wallet:

   ```sh
   bitcoin-cli loadwallet foo
   ```

2. Display any rare ordinals wallet `foo`'s UTXOs:

   ```sh
   ord wallet sats
   ```

### Searching for Rare Ordinals in a Non-Bitcoin Core Wallet

The `ord wallet` command is just a wrapper around Bitcoin Core's RPC API, so to
search for rare ordinals in a non-Bitcoin Core wallet, you'll need to import
your wallet's descriptors into Bitcoin Core.

[Descriptors](https://github.com/bitcoin/bitcoin/blob/master/doc/descriptors.md)
describe the ways that wallets generate private keys and public keys.

You should only import descriptors into Bitcoin Core for your wallet's public
keys, not its private keys.

If your wallet's public key descriptor is compromised, an attacker will be able
to see your wallet's addresses, but your funds will be safe.

If your wallet's private key descriptor is compromised, an attacker can drain
your wallet of funds.

1. Get the wallet descriptor from the wallet whose UTXOs you want to search for
   rare ordinals. It will look something like this:

   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#csvefu29
   ```

2. Create a watch-only wallet named `foo-watch-only`:

   ```sh
   bitcoin-cli createwallet foo-watch-only true true
   ```

   Feel free to give it a better name than `foo-watch-only`!

3. Load the `foo-watch-only` wallet:

   ```sh
   bitcoin-cli loadwallet foo-watch-only
   ```

4. Import your wallet descriptors into `foo-watch-only`:

   ```sh
   bitcoin-cli importdescriptors \
     '[{ "desc": "wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#tpnxnxax", "timestamp":0 }]'
   ```

   If you know the Unix timestamp when your wallet first started receive
   transactions, you may use it for the value of `"timestamp"` instead of `0`.
   This will reduce the time it takes for Bitcoin Core to search for your
   wallet's UTXOs.

5. Check that everything worked:

   ```sh
   bitcoin-cli getwalletinfo
   ```

7. Display your wallet's rare ordinals:

   ```sh
   ord wallet sats
   ```

### Searching for Rare Ordinals in a Wallet that Exports Multi-path Descriptors

Some descriptors describe multiple paths in one descriptor using angle brackets,
e.g., `<0;1>`. Multi-path descriptors are not yet supported by Bitcoin Core, so
you'll first need to convert them into multiple descriptors, and then import
those multiple descriptors into Bitcoin Core.

1. First get the multi-path descriptor from your wallet. It will look something
   like this:

   ```
   wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/<0;1>/*)#fw76ulgt
   ```

2. Create a descriptor for the receive address path:

   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)
   ```

   And the change address path:

   ```
   wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)
   ```

3. Get and note the checksum for the receive address descriptor, in this case
   `tpnxnxax`:

   ```sh
   bitcoin-cli getdescriptorinfo \
     'wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)'
   ```

   ```json
   {
     "descriptor": "wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#csvefu29",
     "checksum": "tpnxnxax",
     "isrange": true,
     "issolvable": true,
     "hasprivatekeys": false
   }
   ```

   And for the change address descriptor, in this case `64k8wnd7`:

   ```sh
   bitcoin-cli getdescriptorinfo \
     'wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)'
   ```

   ```json
   {
     "descriptor": "wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)#fyfc5f6a",
     "checksum": "64k8wnd7",
     "isrange": true,
     "issolvable": true,
     "hasprivatekeys": false
   }
   ```

4. Load the wallet you want to import the descriptors into:

   ```sh
   bitcoin-cli loadwallet foo-watch-only
   ```

4. Now import the descriptors, with the correct checksums, into Bitcoin Core.

   ```sh
   bitcoin-cli \
    importdescriptors \
    '[
      {
        "desc": "wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#tpnxnxax"
        "timestamp":0
      },
      {
        "desc": "wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)#64k8wnd7",
        "timestamp":0
      }
    ]'
   ```

   If you know the Unix timestamp when your wallet first started receive
   transactions, you may use it for the value of the `"timestamp"` fields
   instead of `0`. This will reduce the time it takes for Bitcoin Core to
   search for your wallet's UTXOs.

5. Check that everything worked:

   ```sh
   bitcoin-cli getwalletinfo
   ```

7. Display your wallet's rare ordinals:

   ```sh
   ord wallet sats
   ```

### Exporting Descriptors

#### Sparrow Wallet

Navigate to the `Settings` tab, then to `Script Policy`, and press the edit
button to display the descriptor.

### Transferring Ordinals

The `ord` wallet supports transferring specific satoshis. You can also use
`bitcoin-cli` commands `createrawtransaction`, `signrawtransactionwithwallet`,
and `sendrawtransaction`, how to do so is complex and outside the scope of this
guide.
