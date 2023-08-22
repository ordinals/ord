Inscriptions
============

Inscriptions inscribe sats with arbitrary content, creating bitcoin-native
digital artifacts, more commonly known as NFTs. Inscriptions do not require a
sidechain or separate token.

These inscribed sats can then be transferred using bitcoin transactions, sent
to bitcoin addresses, and held in bitcoin UTXOs. These transactions, addresses,
and UTXOs are normal bitcoin transactions, addresses, and UTXOS in all
respects, with the exception that in order to send individual sats,
transactions must control the order and value of inputs and outputs according
to ordinal theory.

The inscription content model is that of the web. An inscription consists of a
content type, also known as a MIME type, and the content itself, which is a
byte string. This allows inscription content to be returned from a web server,
and for creating HTML inscriptions that use and remix the content of other
inscriptions.

Inscription content is entirely on-chain, stored in taproot script-path spend
scripts. Taproot scripts have very few restrictions on their content, and
additionally receive the witness discount, making inscription content storage
relatively economical.

Since taproot script spends can only be made from existing taproot outputs,
inscriptions are made using a two-phase commit/reveal procedure. First, in the
commit transaction, a taproot output committing to a script containing the
inscription content is created. Second, in the reveal transaction, the output
created by the commit transaction is spent, revealing the inscription content
on-chain.

Inscription content is serialized using data pushes within unexecuted
conditionals, called "envelopes". Envelopes consist of an `OP_FALSE OP_IF …
OP_ENDIF` wrapping any number of data pushes. Because envelopes are effectively
no-ops, they do not change the semantics of the script in which they are
included, and can be combined with any other locking script.

A text inscription containing the string "Hello, world!" is serialized as
follows:

```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_PUSH 1
  OP_PUSH "text/plain;charset=utf-8"
  OP_PUSH 0
  OP_PUSH "Hello, world!"
OP_ENDIF
```

First the string `ord` is pushed, to disambiguate inscriptions from other uses
of envelopes.

`OP_PUSH 1` indicates that the next push contains the content type, and `OP_PUSH
0` indicates that subsequent data pushes contain the content itself. Multiple data
pushes must be used for large inscriptions, as one of taproot's few
restrictions is that individual data pushes may not be larger than 520 bytes.

The inscription content is contained within the input of a reveal transaction,
and the inscription is made on the first sat of its input. This sat can
then be tracked using the familiar rules of ordinal theory, allowing it to be
transferred, bought, sold, lost to fees, and recovered.

Content
-------

The data model of inscriptions is that of a HTTP response, allowing inscription
content to be served by a web server and viewed in a web browser.

Tags
----

Inscriptions can be given extra fields by preceding data pushes with pre-defined
numbers. The most basic inscription has two tags:

`1`: The `content-type` tag where the subsequent data pushes define the MIME type
of the body.

`0`: The `body` tag where the subsequent data pushes contain the actual data.

Furthermore, to give users flexibility and at the same time keep the core
protocol extensible tags are classified into two kinds: `odd` and `even`. Users
can add their own `odd` tags and then clients and explorers can opt-in to show
them. On the other hand `even` tags are reserved for future extensions of the
protocol so any inscription that contains an unknown `even` tag is not
recognized as an inscription and not assigned an inscription ID.

Inscription IDs
---------------

The inscriptions are contained within the inputs of a reveal transaction. In
order to uniquely identify them they are assigned an ID of the form
`521f8eccffa4c41a3a7728dd012ea5a4a02feed81f41159231251ecf1e5c79dai0`, where the
part in front of the `i` is the transaction ID (`txid`) of the reveal
transaction. The last number defines the index (starting at 0) of new inscriptions
being inscribed in the reveal transaction.

Inscriptions can either be located in different inputs, within the same input or
a combination of both. In any case the ordering is clear, since a parser would
go through the inputs consecutively and look for all inscription `envelopes`.

Sandboxing
----------

HTML and SVG inscriptions are sandboxed in order to prevent references to
off-chain content, thus keeping inscriptions immutable and self-contained.

This is accomplished by loading HTML and SVG inscriptions inside `iframes` with
the `sandbox` attribute, as well as serving inscription content with
`Content-Security-Policy` headers.
