URIs
====

*This document is a draft. It should be considered provisional and subject to
change at any time. The `ord:` schema has not been registered with the IANA.*

Inscriptions content can be addressed with inscription URIs using the `ord:`
schema.

Inscription URIs consist of `ord:` followed by a target inscription ID. `ord:`
is not followed by `//`, since the schema-specific part of inscription URIs,
namely the target inscription ID, does not contain a hierarchical structure.

For example, the inscription URI of the genesis inscription is:

```
ord:6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0
```

Inscription URIs match the following verbose regular expression:

```
(?i)            # case-insensitive
ord:            # schema
[0-9a-f]{64}    # transaction ID
i               # separator
(0|[1-9][0-9]*) # inscription index
```

Inscription URIs are case-insensitive and can thus use the more compact
alphanumeric mode when encoded as QR codes. Lowercase is, however, the
preferred presentation style.

The referent of an inscription URI is an HTTP resource with the content,
content type, content encoding, and content length corresponding to the
inscription with the given ID.

The referent of an inscription URI is always the original content of the target
inscription, and not the content of the delegate, regardless of whether or not
the target inscription has a delegate.
