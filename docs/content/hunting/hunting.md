Ordinal Hunting 101
===================

Tools for the hunt
-------------------

- `bitcoind -txindex=1` fully synced to mainnet
- `bitcoin-cli` 
- `ord` compiled and indexed
- wallet (containing your UTXOs) that supports exporting a [descriptor](https://github.com/bitcoin/bitcoin/blob/master/doc/descriptors.md)


Steps
-----

1. Create a blank wallet without any private keys and load it into bitcoind
```bash
bitcoin-cli createwallet ord-watch-only true true
bitcoin-cli loadwallet ord-watch-only 
```

2. Get the descriptor from your wallet of choice. In Sparrow wallet you can find
that in the `Settings` tab, then at `Script Policy` and the press the edit button.
The descriptor should look something like this:
```bash
wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/<0;1>/*)#fw76ulgt
```

3. Some wallets use a descriptor encoding that unfortunately hasn't been 
[merged](https://github.com/bitcoin/bitcoin/pull/22838) into Bitcoin Core yet, 
so some extra steps are necessary. First we need the descriptor checksum for the
receive addresses (`/0/*`):
```bash
bitcoin-cli getdescriptorinfo 'wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)'
{
  "descriptor": "wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#csvefu29",
  "checksum": "tpnxnxax",
  "isrange": true,
  "issolvable": true,
  "hasprivatekeys": false
}
```
And then for the change addresses (`/1/*`):
```bash 
bitcoin-cli getdescriptorinfo 'wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)'
{
  "descriptor": "wpkh([bf1dd55e/84'/0'/0']xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)#fyfc5f6a",
  "checksum": "64k8wnd7",
  "isrange": true,
  "issolvable": true,
  "hasprivatekeys": false
}
```

4. With these checksums we can then import the descriptors (append #checksum to the end) Bitcoind needs to know how far back to look for transactions so find out when the first transaction took place and convert that to unix time and put it in the timestamp field. This command can take quite a while to complete.
```bash
bitcoin-cli importdescriptors '[{ "desc": "wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/0/*)#tpnxnxax", "timestamp":1455191478 } {"desc":"wpkh([bf1dd55e/84h/0h/0h]xpub6CcJtWcvFQaMo39ANFi1MyXkEXM8T8ZhnxMtSjQAdPmVSTHYnc8Hwoc11VpuP8cb8JUTboZB5A7YYGDonYySij4XTawL6iNZvmZwdnSEEep/1/*)#64k8wnd7" , "timestamp":1455191478 }]'
```

5. Test that everthing worked with:
```bash
bitcoin-cli getwalletinfo
```

6. Now that we have a wallet loaded we can use the `identify` command, which will list and rare ordinals contained in your UTXOs.
```bash
ord wallet identify 
```
