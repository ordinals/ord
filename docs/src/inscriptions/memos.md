Memos
=====

Memos allow the owner of an inscription to attach append-only data to that
inscription. A memo is itself an inscription, linked to its target via tag `27`,
and burned on creation by sending it to an `OP_RETURN` output.

Because the memo is burned, it does not remain in the creator's wallet, keeping
things clean. The content remains permanently readable on-chain.

Memos are completely independent of parent-child relationships. An inscription
can have both children and memos, and a memo can itself be a child of another
inscription if desired.

### Specification

To create a memo M targeting inscription T:

- Create an inscribe transaction as usual for M.
- Spend the target T in one of the inputs, and return it to its owner in one of
  the outputs (same mechanic as parent-child).
- Include tag `27`, i.e. `OP_PUSH 27`, in M, with the value of the serialized
  binary inscription ID of T, serialized as the 32-byte `TXID`, followed by the
  four-byte little-endian `INDEX`, with trailing zeroes omitted.
- Send the memo inscription to an `OP_RETURN` output with a value of 1 sat.

The memo inscription receives the "burned" charm.

### Example

An example of a memo targeting
`000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fi0`:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 1
  OP_PUSH "application/json"
  OP_PUSH 27
  OP_PUSH 0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100
  OP_PUSH 0
  OP_PUSH '{"type":"comment","text":"Hello from a memo"}'
OP_ENDIF
```

Note that the value of tag `27` is binary, not hex, and that the target
inscription must be spent as one of the inputs of the inscribe transaction.

### CLI

```
ord wallet inscribe --fee-rate <FEE_RATE> --memo <INSCRIPTION_ID> --file <FILE>
```

The `--memo` flag cannot be combined with `--parent`.

### Delegate Memos

A memo can use `--delegate` instead of `--file` to point to an existing
inscription for its content:

```
ord wallet inscribe --fee-rate <FEE_RATE> --memo <INSCRIPTION_ID> --delegate <DELEGATE_ID>
```

The memo itself carries no content bytes - it resolves through the delegate
when accessed via `/r/memos/.../content`. This is useful for applications with
large or repeated configuration payloads. For example, an app with complex
template settings could define each configuration once as an inscription, then
apply or re-apply those settings as delegate memos without paying for the
content bytes each time.

### Notes

Tag `27` is an odd tag, so memos are backwards compatible with older versions
of `ord` that do not recognize the tag.

The memo's content can be any content type, just like any other inscription.
Text, JSON, images, or any other media are all valid memo content. Memos can
also carry metadata via tag `5`, allowing a single memo to combine a media body
(e.g. an image) with structured data (e.g. JSON metadata encoded as CBOR).

Memos for an inscription are visible on the target inscription's page in the
explorer, and are accessible via the recursive endpoints documented in
[recursion](recursion.md).
