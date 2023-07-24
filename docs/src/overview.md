Ordinal Theory Overview
=======================

Ordinals are a numbering scheme for satoshis that allows tracking and
transferring individual sats. These numbers are called [ordinal
numbers](https://ordinals.com). Satoshis are numbered in the order in which
they're mined, and transferred from transaction inputs to transaction outputs
first-in-first-out. Both the numbering scheme and the transfer scheme rely on
*order*, the numbering scheme on the *order* in which satoshis are mined, and
the transfer scheme on the *order* of transaction inputs and outputs. Thus the
name, *ordinals*.

Technical details are available in [the
BIP](https://github.com/ordinals/ord/blob/master/bip.mediawiki).

Ordinal theory does not require a separate token, another blockchain, or any
changes to Bitcoin. It works right now.

Ordinal numbers have a few different representations:

- *Integer notation*:
  [`2099994106992659`](https://ordinals.com/sat/2099994106992659) The
  ordinal number, assigned according to the order in which the satoshi was
  mined.

- *Decimal notation*:
  [`3891094.16797`](https://ordinals.com/sat/3891094.16797) The first
  number is the block height in which the satoshi was mined, the second the
  offset of the satoshi within the block.

- *Degree notation*:
  [`3°111094′214″16797‴`](https://ordinals.com/sat/3%C2%B0111094%E2%80%B2214%E2%80%B316797%E2%80%B4).
  We'll get to that in a moment.

- *Percentile notation*:
  [`99.99971949060254%`](https://ordinals.com/sat/99.99971949060254%25) .
  The satoshi's position in Bitcoin's supply, expressed as a percentage.

- *Name*: [`satoshi`](https://ordinals.com/sat/satoshi). An encoding of the
  ordinal number using the characters `a` through `z`.

Arbitrary assets, such as NFTs, security tokens, accounts, or stablecoins can
be attached to satoshis using ordinal numbers as stable identifiers.

Ordinals is an open-source project, developed [on
GitHub](https://github.com/ordinals/ord). The project consists of a BIP describing
the ordinal scheme, an index that communicates with a Bitcoin Core node to
track the location of all satoshis, a wallet that allows making ordinal-aware
transactions, a block explorer for interactive exploration of the blockchain,
functionality for inscribing satoshis with digital artifacts, and this manual.

Rarity
------

Humans are collectors, and since satoshis can now be tracked and transferred,
people will naturally want to collect them. Ordinal theorists can decide for
themselves which sats are rare and desirable, but there are some hints…

Bitcoin has periodic events, some frequent, some more uncommon, and these
naturally lend themselves to a system of rarity. These periodic events are:

- *Blocks*: A new block is mined approximately every 10 minutes, from now until
  the end of time.

- *Difficulty adjustments*: Every 2016 blocks, or approximately every two
  weeks, the Bitcoin network responds to changes in hashrate by adjusting the
  difficulty target which blocks must meet in order to be accepted.

- *Halvings*: Every 210,000 blocks, or roughly every four years, the amount of
  new sats created in every block is cut in half.

- *Cycles*: Every six halvings, something magical happens: the halving and the
  difficulty adjustment coincide. This is called a conjunction, and the time
  period between conjunctions a cycle. A conjunction occurs roughly every 24
  years. The first conjunction should happen some time in 2032.

This gives us the following rarity levels:

- `common`: Any sat that is not the first sat of its block
- `uncommon`: The first sat of each block
- `rare`: The first sat of each difficulty adjustment period
- `epic`: The first sat of each halving epoch
- `legendary`: The first sat of each cycle
- `mythic`: The first sat of the genesis block

Which brings us to degree notation, which unambiguously represents an ordinal
number in a way that makes the rarity of a satoshi easy to see at a glance:

```
A°B′C″D‴
│ │ │ ╰─ Index of sat in the block
│ │ ╰─── Index of block in difficulty adjustment period
│ ╰───── Index of block in halving epoch
╰─────── Cycle, numbered starting from 0
```

Ordinal theorists often use the terms "hour", "minute", "second", and "third"
for *A*, *B*, *C*, and *D*, respectively.

Now for some examples. This satoshi is common:

```
1°1′1″1‴
│ │ │ ╰─ Not first sat in block
│ │ ╰─── Not first block in difficutly adjustment period
│ ╰───── Not first block in halving epoch
╰─────── Second cycle
```


This satoshi is uncommon:

```
1°1′1″0‴
│ │ │ ╰─ First sat in block
│ │ ╰─── Not first block in difficutly adjustment period
│ ╰───── Not first block in halving epoch
╰─────── Second cycle
```

This satoshi is rare:

```
1°1′0″0‴
│ │ │ ╰─ First sat in block
│ │ ╰─── First block in difficulty adjustment period
│ ╰───── Not the first block in halving epoch
╰─────── Second cycle
```

This satoshi is epic:

```
1°0′1″0‴
│ │ │ ╰─ First sat in block
│ │ ╰─── Not first block in difficulty adjustment period
│ ╰───── First block in halving epoch
╰─────── Second cycle
```

This satoshi is legendary:

```
1°0′0″0‴
│ │ │ ╰─ First sat in block
│ │ ╰─── First block in difficulty adjustment period
│ ╰───── First block in halving epoch
╰─────── Second cycle
```

And this satoshi is mythic:

```
0°0′0″0‴
│ │ │ ╰─ First sat in block
│ │ ╰─── First block in difficulty adjustment period
│ ╰───── First block in halving epoch
╰─────── First cycle
```

If the block offset is zero, it may be omitted. This is the uncommon satoshi
from above:

```
1°1′1″
│ │ ╰─ Not first block in difficutly adjustment period
│ ╰─── Not first block in halving epoch
╰───── Second cycle
```

Rare Satoshi Supply
-------------------

### Total Supply

- `common`: 2.1 quadrillion
- `uncommon`: 6,929,999
- `rare`: 3437
- `epic`: 32
- `legendary`: 5
- `mythic`: 1

### Current Supply

- `common`: 1.9 quadrillion
- `uncommon`: 745,855
- `rare`: 369
- `epic`: 3
- `legendary`: 0
- `mythic`: 1

At the moment, even uncommon satoshis are quite rare. As of this writing,
745,855 uncommon satoshis have been mined - one per 25.6 bitcoin in
circulation.

Names
-----

Each satoshi has a name, consisting of the letters *A* through *Z*, that get
shorter the further into the future the satoshi was mined. They could start
short and get longer, but then all the good, short names would be trapped in
the unspendable genesis block.

As an example, 1905530482684727°'s name is "iaiufjszmoba". The name of the last
satoshi to be mined is "a". Every combination of 10 characters or less is out
there, or will be out there, some day.

Exotics
-------

Satoshis may be prized for reasons other than their name or rarity. This might
be due to a quality of the number itself, like having an integer square or cube
root. Or it might be due to a connection to a historical event, such as
satoshis from block 477,120, the block in which SegWit activated, or
2099999997689999°, the last satoshi that will ever be mined.

Such satoshis are termed "exotic". Which satoshis are exotic and what makes
them so is subjective. Ordinal theorists are encouraged to seek out exotics
based on criteria of their own devising.

Inscriptions
------------

Satoshis can be inscribed with arbitrary content, creating Bitcoin-native
digital artifacts. Inscribing is done by sending the satoshi to be inscribed in
a transaction that reveals the inscription content on-chain. This content is
then inextricably linked to that satoshi, turning it into an immutable digital
artifact that can be tracked, transferred, hoarded, bought, sold, lost, and
rediscovered.

Archaeology
-----------

A lively community of archaeologists devoted to cataloging and collecting early
NFTs has sprung up. [Here's a great summary of historical NFTs by
Chainleft.](https://mirror.xyz/chainleft.eth/MzPWRsesC9mQflxlLo-N29oF4iwCgX3lacrvaG9Kjko)

A commonly accepted cut-off for early NFTs is March 19th, 2018, the date the
first ERC-721 contract, [SU SQUARES](https://tenthousandsu.com/), was deployed
on Ethereum.

Whether or not ordinals are of interest to NFT archaeologists is an open
question! In one sense, ordinals were created in early 2022, when the Ordinals
specification was finalized. In this sense, they are not of historical
interest.

In another sense though, ordinals were in fact created by Satoshi Nakamoto in
2009 when he mined the Bitcoin genesis block. In this sense, ordinals, and
especially early ordinals, are certainly of historical interest.

Many ordinal theorists favor the latter view. This is not least because the
ordinals were independently discovered on at least two separate occasions, long
before the era of modern NFTs began.

On August 21st, 2012, Charlie Lee [posted a proposal to add proof-of-stake to
Bitcoin to the Bitocin Talk
forum](https://bitcointalk.org/index.php?topic=102355.0). This wasn't an asset
scheme, but did use the ordinal algorithm, and was implemented but never
deployed.

On October 8th, 2012, jl2012 [posted a scheme to the same
forum](https://bitcointalk.org/index.php?topic=117224.0) which uses decimal
notation and has all the important properties of ordinals. The scheme was
discussed but never implemented.

These independent inventions of ordinals indicate in some way that ordinals
were discovered, or rediscovered, and not invented. The ordinals are an
inevitability of the mathematics of Bitcoin, stemming not from their modern
documentation, but from their ancient genesis. They are the culmination of a
sequence of events set in motion with the mining of the first block, so many
years ago.
