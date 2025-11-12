Properties
==========

Inscriptions may include [CBOR](https://cbor.io/) properties, stored as data
pushes in fields with tag `17`. Since data pushes are limited to 520 bytes,
CBOR longer than 520 bytes must be split into multiple tag `17` fields which
are then concatenated before decoding.

Properties are a structured counterpart to [metadata](metadata.md). While
metadata may contain arbitrary CBOR which has no protocol-defined meaning, and
is presented on `/inscription` as free-form HTML, properties have
protocol-defined meaning and must conform to a strict schema.

Indefinite-length CBOR types are not supported in properties. All maps, arrays,
byte strings, and text strings must be definite.

The non-normative [CDDL](https://datatracker.ietf.org/doc/html/rfc8610) schema
of the properties value is as follows:

```cddl
Properties = {
  ? 0: [*Item],
  ? 1: Attributes,
  * any => any,
}

Item = {
  ? 0: bytes .size (32..36),
  ? 1: Attributes,
  * any => any,
}

Attributes = {
  ? 0: text,   ; title
  ? 1: Traits, ; traits
  * any => any,
}

Traits = {
  ; trait names must be unique
  text => Trait,
}

Trait = bool / int / null / text;
```

The above CDDL schema is provided as a convenience. As always, the ordinals
reference implementation `ord` is the normative specification of inscriptions,
and thus the properties field.

Fields matching the `* any => any` wildcard must be accepted but ignored, for
compatibility with future additions.

Galleries
=========

Inscriptions whose properties field contains `Item`s are galleries.

Galleries contain `Item`s, whose key `0` contains a serialized inscription ID.
Inscription ID `TXIDiINDEX` is serialized as a byte string containing the 32
byte `TXID`, concatenated with by the four-byte little-endian `INDEX`. Trailing
zeros may be removed from four-byte `INDEX`, so IDs ending in `i0` may be
serialized in 32 bytes.

Gallery items are displayed on the inscriptions `/inscription` page on the
explorer.

Galleries are similar to children, in that they provide a way to create
collections of inscriptions. However, galleries are permissionless. Anyone may
create a gallery including any inscriptions. Thus, inclusion in a gallery does
not imply provenance. Additionally, because of this, inclusion in a gallery
does not create a backlink from the gallery item's `/inscription` page to the
gallery.

Galleries may be created when batch inscribing with `ord wallet batch` by
including an array of string inscription IDs of under the `gallery` key of the
inscription entry in the batch file, or when using `ord wallet inscribe` using
the `--gallery` option.

Attributes
==========

Attributes are structured metadata that may be attached to both inscriptions
and gallery items.

Traits
------

Traits are a structured counterpart to metadata. While metadata may be
arbitrary CBOR, traits are ordered maps of string names to values. Names must
be unique, and values may be booleans, integers, `null`, or strings.
