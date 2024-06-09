Delegate
========

Inscriptions may nominate a delegate inscription. Requests for the content of
an inscription with a delegate will instead return the content, content type 
and content encoding of the delegate. This can be used to cheaply create copies 
of an inscription.

### Specification

To create an inscription I with delegate inscription D:

- Create an inscription D. Note that inscription D does not have to exist when
  making inscription I. It may be inscribed later. Before inscription D is
  inscribed, requests for the content of inscription I will return a 404.
- Include tag `11`, i.e. `OP_PUSH 11`, in I, with the value of the serialized
  binary inscription ID of D, serialized as the 32-byte `TXID`, followed by the
  four-byte little-endian `INDEX`, with trailing zeroes omitted.

_NB_ The bytes of a bitcoin transaction ID are reversed in their text
representation, so the serialized transaction ID will be in the opposite order.

### Example

An example of an inscription which delegates to
`000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1fi0`:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 11
  OP_PUSH 0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100
OP_ENDIF
```

Note that the value of tag `11` is decimal, not hex.

The delegate field value uses the same encoding as the parent field. See
[provenance](provenance.md) for more examples of inscription ID encodings

### On-chain Examples

* The [Oscillations *](https://gamma.io/ordinals/collections/oscillations-mdv) collection utilizes delegation, provenance, recursion, sat-endpoint, and detects the kind of sat that each piece is inscribed on (sattribute-aware). Each piece is a delegate of [this inscription](https://ordinals.com/inscription/52b4ea10c2518c954c73594e403ccfb2d50044f5a3b09a224dfa3bf06dd1d499i0).
* [This inscription](https://ordinals.com/inscription/23a8f17fff4a73e2932dfc76e46d14d4f8975da96f5d5ae9a45898422056071ai0) was inscribed as a delegate of [this inscription](https://ordinals.com/inscription/9ff39db4c51f831225d41efbd29a399f2b16c758970ec4ab95a1a17e8be59088i0) and is also the parent inscription of a rune.
