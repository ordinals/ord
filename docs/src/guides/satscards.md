Satscard
========

[Satscards](https://satscard.com/) are cards which can be used to store
bitcoin, inscriptions, and runes.

Slots
-----

Each satscard has ten slots containing private keys with corresponding bitcoin
addresses.

Initially, all slots are sealed and the private keys are stored only the
satscard.

Slots can be unsealed, which allows the corresponding private key to be
extracted.

Unsealing is permanent. If a satscard is sealed, you can have some confidence
that private key is not known to anyone. That taking physical ownership of a
satscard makes you the sole owner of assets in any sealed slots.

Lifespan
--------

Satscards are expected to have a usable lifetime of ten years. Do not use
satscards for long-term storage of valuable assets.


Viewing
-------

When placed on a smartphone, the satscard transmits a URL, beginning with
`https://satscard.com/start` or `https://getsatscard.com/start`, depending on
when it was manufactured.

This URL contains a signature which can be used to recover the address of the
current slot. This signature is made over a random nonce, so it changes every
time the satscard is tapped, and provides some confidence that the satscard
contains the private key.

`ord` supports viewing the contents of a satscard by entering the full URL into
the `ord` explorer search bar, or the input field on the `/satscard` page.

For `ordinals.com`, this is
[ordinals.com/satscard](https://ordinals.com/satscard).

Unsealing
---------

Satscard slots can be unsealed and the private keys extracted using the `cktap`
binary, available in the
[coinkite-tap-proto](https://github.com/coinkite/coinkite-tap-proto)
repository.

After a satscard slot is unsealed, all assets should be swept from that slot to
another wallet, as the private key can now be read via NFC with the PIN.

Sweeping
--------

First, clone the
[coinkite-tap-proto](https://github.com/coinkite/coinkite-tap-proto) repository
and install the `cktap` binary inside a Python virtual environment:

```console
$ git clone https://github.com/coinkite/coinkite-tap-proto.git
$ cd coinkite-tap-proto
$ python3 -m venv venv
$ source venv/bin/activate
$ pip3 install 'coinkite-tap-protocol[cli]'
```

Unseal the satscard slot:

```console
$ cktap unseal
```

Export the private key:

```console
$ cktap wif
```

This will print something like:

```
Slot #1:

bc1q6e2renlnjyhwnyncd7sgr6s4s2cnkdknvrz40d

p2wpkh:KwVN7HkFnTymkfCh1b4Y5bRGftySuK4jjPQdgBoMURWCi8CjbZPN
```

Note the address type, in this case `p2wpkh`, and the private key, in this case
`KwVN7HkFnTymkfCh1b4Y5bRGftySuK4jjPQdgBoMURWCi8CjbZPN`.

Write the private key to a file. This example will use the filename
`private-key.wif`.

Now sweep the assets into your `ord` wallet:

```console
$ cat private-key.wif | ord wallet sweep --address-type p2wpkh --fee-rate 10
```

Be careful, and good luck!
