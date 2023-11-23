Recursion
=========

An important exception to [sandboxing](../inscriptions.md#sandboxing) is
recursion: access to `ord`'s `/content` endpoint is permitted, allowing
inscriptions to access the content of other inscriptions by requesting
`/content/<INSCRIPTION_ID>`.

This has a number of interesting use-cases:

- Remixing the content of existing inscriptions.

- Publishing snippets of code, images, audio, or stylesheets as shared public
  resources.

- Generative art collections where an algorithm is inscribed as JavaScript,
  and instantiated from multiple inscriptions with unique seeds.

- Generative profile picture collections where accessories and attributes are
  inscribed as individual images, or in a shared texture atlas, and then
  combined, collage-style, in unique combinations in multiple inscriptions.

The following other recursive endpoints return JSON:

- `/r/blockheight`: latest block height.
- `/r/blockhash`: latest block hash.
- `/r/blockhash/<HEIGHT>`: block hash at given block height.
- `/r/blocktime`: UNIX time stamp of latest block.
- `/r/metadata/<INSCRIPTION_ID>`: returns a JSON string containing the hex-encoded CBOR metadata.

For backwards compatibility these additional endpoints are supported.

- `/blockheight`: latest block height.
- `/blockhash`: latest block hash.
- `/blockhash/<HEIGHT>`: block hash at given block height.
- `/blocktime`: UNIX time stamp of latest block.
