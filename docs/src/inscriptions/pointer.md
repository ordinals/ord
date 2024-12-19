Pointer
=======

In order to make an inscription on a sat other than the first of its input, a
zero-based integer, called the "pointer", can be provided with tag `2`, causing
the inscription to be made on the sat at the given position in the outputs. If
the pointer is equal to or greater than the number of total sats in the outputs
of the inscribe transaction, it is ignored, and the inscription is made as
usual. The value of the pointer field is a little endian integer, with trailing
zeroes ignored.

An even tag is used, so that old versions of `ord` consider the inscription to
be unbound, instead of assigning it, incorrectly, to the first sat.

This can be used to create multiple inscriptions in a single transaction on
different sats, when otherwise they would be made on the same sat.

Examples
--------

An inscription with pointer 255:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 1
  OP_PUSH "text/plain;charset=utf-8"
  OP_PUSH 2
  OP_PUSH 0xff
  OP_PUSH 0
  OP_PUSH "Hello, world!"
OP_ENDIF
```

An inscription with pointer 256:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 1
  OP_PUSH "text/plain;charset=utf-8"
  OP_PUSH 2
  OP_PUSH 0x0001
  OP_PUSH 0
  OP_PUSH "Hello, world!"
OP_ENDIF
```

An inscription with pointer 256, with trailing zeroes, which are ignored:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 1
  OP_PUSH "text/plain;charset=utf-8"
  OP_PUSH 2
  OP_PUSH 0x000100
  OP_PUSH 0
  OP_PUSH "Hello, world!"
OP_ENDIF
```
