Provenance
==========

The owner of an inscription can create child inscriptions, trustlessly
establishing the provenance of those children on-chain as having been created
by the owner of the parent inscription. This can be used for collections, with
the children of a parent inscription being members of the same collection.

Children can themselves have children, allowing for complex hierarchies. For
example, an artist might create an inscription representing themselves, with
sub inscriptions representing collections that they create, with the children
of those sub inscriptions being items in those collections.

### Specification

To create a child inscription C with parent inscription P:

- Create an inscribe transaction T as usual for C.
- Spend the parent P in one of the inputs of T.
- Include tag `3`, i.e. `OP_PUSH 3`, in C, with the value of the serialized
  binary inscription ID of P, serialized as the 32-byte `TXID`, followed by the
  four-byte little-endian `INDEX`, with trailing zeroes omitted.

_NB_ The bytes of a bitcoin transaction ID are reversed in their text
representation, so the serialized transaction ID will be in the opposite order.

### Example

An example of a child inscription of
`000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fi0`:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 1
  OP_PUSH "text/plain;charset=utf-8"
  OP_PUSH 3
  OP_PUSH 0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100
  OP_PUSH 0
  OP_PUSH "Hello, world!"
OP_ENDIF
```

Note that the value of tag `3` is binary, not hex, and that for the child
inscription to be recognized as a child,
`000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fi0` must be
spent as one of the inputs of the inscribe transaction.

Example encoding of inscription ID
`000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fi255`:

```
OP_FALSE
OP_IF
  …
  OP_PUSH 3
  OP_PUSH 0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100ff
  …
OP_ENDIF
```

And of inscription ID `000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fi256`:

```
OP_FALSE
OP_IF
  …
  OP_PUSH 3
  OP_PUSH 0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a090807060504030201000001
  …
OP_ENDIF
```

### Notes

The tag `3` is used because it is the first available odd tag. Unrecognized odd
tags do not make an inscription unbound, so child inscriptions would be
recognized and tracked by old versions of `ord`.

A collection can be closed by burning the collection's parent inscription,
which guarantees that no more items in the collection can be issued.


See
[examples](examples.md#provenance) for on-chain examples of inscriptions that feature this functionality.