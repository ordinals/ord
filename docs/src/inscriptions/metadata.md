Metadata
========

Inscriptions may include CBOR metadata, stored as data pushes in fields with
tag `5`. Tapscript data pushes are limited to 520 bytes. Metadata longer than
520 bytes may be split into multiple tag `5` fields, which will then be
concatinated before decoding.

Metadata is human readable, and all metadata will be displayed to the user with
its inscription. Inscribers are encouraged to consider how metadata will be
displayed, and make metadata concise and attractive.

Metadata is rendered to HTML for display as follows:

- `null`, `true`, `false`, numbers, and strings are rendered as plain text.
- Arrays are rendered as `<ul>` tags, with every element wrapped in `<li>`
  tags.
- Objects are rendered as `<dl>` tags, with every key wrapped in `<dt>` tags,
  and every value wrapped in `<dd>` tags.

Example
-------

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

Which would then be concatinated into
`{"very":"long","metadata":"is","finally":"done"}`.
