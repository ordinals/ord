Teleburning
===========

Teleburn addresses can be used to burn assets on other blockchains, leaving
behind in the smoking rubble a sort of forwarding address pointing to an
inscription on Bitcoin.

Teleburning an asset means something like, "I'm out. Find me on Bitcoin."

Teleburn addresses are derived from inscription IDs. They have no corresponding
private key, so assets sent to a teleburn address are burned. Currently, only
Ethereum teleburn addresses are supported. Pull requests adding teleburn
addresses for other chains are welcome.

Ethereum
--------

Ethereum teleburn addresses are derived by taking the first 20 bytes of the
SHA-256 hash of the inscription ID, serialized as 36 bytes, with the first 32
bytes containing the transaction ID, and the last four bytes containing
big-endian inscription index, and interpreting it as an Ethereum address.

Example
-------

The ENS domain name [rodarmor.eth](https://app.ens.domains/rodarmor.eth), was
teleburned to [inscription
zero](https://ordinals.com/inscription/6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0).

Running the inscription ID of inscription zero is
`6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0`.

Passing `6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0` to
the teleburn command:

```bash
$ ord teleburn 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0
```

Returns:

```json
{
  "ethereum": "0xe43A06530BdF8A4e067581f48Fae3b535559dA9e"
}
```

Indicating that `0xe43A06530BdF8A4e067581f48Fae3b535559dA9e` is the Ethereum
teleburn address for inscription zero, which is, indeed, the current owner, on
Ethereum, of `rodarmor.eth`.
