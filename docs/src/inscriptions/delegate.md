Delegate
========

Inscriptions may nominate a delegate inscription. Requests for the content of
an inscription with a delegate will instead return the content and content type
of the delegate. This can be used to cheaply create copies of an inscription.

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
[provenance](provenance.md) for more examples of inscrpition ID encodings;
