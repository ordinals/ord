Batch Inscriptions
==================

Using the [pointer field](./pointer.md) it is now possible to do batch
inscriptions. This is especially helpful for collections, since the parent can
be passed into a transaction and have multiple children at once.   There is a lot
that can be expanded in functionality like setting destination individually,
adding more modes like `reinscribe` and `consecutive-sats` or setting a custom
postage.

To batch inscribe do the following:
```bash
ord wallet inscribe --fee-rate 21 --batch batch.yaml
```

Example `batch.yaml`
--------------------

```yaml
# There are two modes for now:
# `separate-outputs`: all inscriptions in separate outputs with an output size of 10000sat.
# `shared-output`: all inscriptions in same output separated by the postage (10000sat).
mode: separate-outputs

# Specify parent for all inscriptions.
parent: 6ac5cacb768794f4fd7a78bf00f2074891fce68bd65c4ff36e77177237aacacai0

# List of inscriptions with info.
# This will be the order in which they will be inscribed.
batch:
  - inscription: ./mango.avif
    metadata:
      title: Delicious Mangos
      description: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam semper, ligula ornare laoreet tincidunt, odio nisi euismod tortor, vel blandit metus est et odio. Nullam venenatis, urna et molestie vestibulum, orci mi efficitur risus, eu malesuada diam lorem sed velit. Nam fermentum dolor et luctus euismod.

  - inscription: ./token.json
    metaprotocol: brc-20

  - inscription: ./tulip.png
    metadata:
      author: Satoshi Nakamoto
```
