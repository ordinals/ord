Batch Inscriptions
==================

Using the [pointer field](./pointer.md) it is now possible to do batch
inscriptions. This is especially helpful for collections, since the parent can
be passed into a transaction and have multiple children at once. There is a lot
that can be expanded in functionality but for now all children land in the same
output, separated by 10000 sats. You split them out with the send command. The
next iteration of this command will put all inscriptions into separate outputs
for easier sending and even allow setting the destination for each individual
inscription.

Example
-------

```yaml
# Example batch file

# For now there is only one mode that inscribes all inscription into the same
# output seperated by the postage (10000sat). They can be peeled off one by
# one by using `ord wallet send` (last out first).
mode: shared-output

# Specify parent for all inscriptions
# In the future this can be set for each inscription individually
parent: 6ac5cacb768794f4fd7a78bf00f2074891fce68bd65c4ff36e77177237aacacai0

# List of inscriptions with info.
# This will be the order in which they will be inscribed
batch:
  - inscription: ./mango.avif

  - inscription: ./token.json
    metaprotocol: brc-20
    metadata: ./token-metadata.json
```
