Metadata
========

Inscriptions may include [CBOR](https://cbor.io/) metadata, stored as data
pushes in fields with tag `5`. Since data pushes are limited to 520 bytes,
metadata longer than 520 bytes must be split into multiple tag `5` fields,
which will then be concatenated before decoding.

Metadata is human readable, and all metadata will be displayed to the user with
its inscription. Inscribers are encouraged to consider how metadata will be
displayed, and make metadata concise and attractive.

Metadata is rendered to HTML for display as follows:

- `null`, `true`, `false`, numbers, floats, and strings are rendered as plain
  text.
- Byte strings are rendered as uppercase hexadecimal.
- Arrays are rendered as `<ul>` tags, with every element wrapped in `<li>`
  tags.
- Maps are rendered as `<dl>` tags, with every key wrapped in `<dt>` tags, and
  every value wrapped in `<dd>` tags.
- Tags are rendered as the tag , enclosed in a `<sup>` tag, followed by the
  value.

CBOR is a complex spec with many different data types, and multiple ways of
representing the same data. Exotic data types, such as tags, floats, and
bignums, and encoding such as indefinite values, may fail to display correctly
or at all. Contributions to `ord` to remedy this are welcome.

Example
-------

Since CBOR is not human readable, in these examples it is represented as JSON.
Keep in mind that this is *only* for these examples, and JSON metadata will
*not* be displayed correctly.

The metadata `{"foo":"bar","baz":[null,true,false,0]}` would be included in an inscription as:

```
OP_FALSE
OP_IF
    ...
    OP_PUSH 0x05 OP_PUSH '{"foo":"bar","baz":[null,true,false,0]}'
    ...
OP_ENDIF
```

And rendered as:

```
<dl>
  ...
  <dt>metadata</dt>
  <dd>
    <dl>
      <dt>foo</dt>
      <dd>bar</dd>
      <dt>baz</dt>
      <dd>
        <ul>
          <li>null</li>
          <li>true</li>
          <li>false</li>
          <li>0</li>
        </ul>
      </dd>
    </dl>
  </dd>
  ...
</dl>
```

Metadata longer than 520 bytes must be split into multiple fields:

```
OP_FALSE
OP_IF
    ...
    OP_PUSH 0x05 OP_PUSH '{"very":"long","metadata":'
    OP_PUSH 0x05 OP_PUSH '"is","finally":"done"}'
    ...
OP_ENDIF
```

Which would then be concatenated into
`{"very":"long","metadata":"is","finally":"done"}`.
