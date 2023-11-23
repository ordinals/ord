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

A few other endpoints that inscriptions may access are the following:

- `/blockheight`: latest block height.
- `/blockhash`: latest block hash.
- `/blockhash/<HEIGHT>`: block hash at given block height.
- `/blocktime`: UNIX time stamp of latest block.
