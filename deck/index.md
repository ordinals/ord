# Ordinals

# Today's Agenda
- Who am I?
- What are ordinals?
- What does that have to do with NFTs?

---

# Feel free to interrupt and ask questions!

---

# Who am I?
- I'm just this guy, you know?
- Bitcoin, Rust, and generative art programmer
- Last big project was Agora, a server for selling downloads for Lightning
  Network payments

---

# Why ordinals?
- Wanted a simple protocol for assets on Bitcoin
- Don't require any modification to the protocol
- Aesthetically appealing, so particularly well-suited for art

---

# What are ordinals?

---

# Ordinals are just serial numbers for satoshis

---

# They start at 0, and go up to 1,906,077,499,999,999 (so far!)

---

# Are transferred with a simple first-in-first-out algorithm

```
[2] [1] [3] → [4] [2]
```

```
[a b] [c] [d e f] → [? ? ? ?] [? ?]
```

```
[a b] [c] [d e f] → [a b c d] [e f]
```

---

# What about fees?

```
[2] [1] [3] → [4]
```

```
[a b] [c] [d e f] → [a b c d]
```

```
[SUBSIDY] [e f] → [SUBSIDY e f]
```

---

# Specification

```python
# subsidy of block at given height
def subsidy(height):
  return 50 * 100_000_000 >> height // 210_000

# first ordinal of subsidy of block at given height
def first_ordinal(height):
  start = 0
  for height in range(height):
    start += subsidy(height)
  return start

# assign ordinals in given block
def assign_ordinals(block):
  first = first_ordinal(block.height)
  last = first + subsidy(block.height)
  coinbase_ordinals = list(range(first, last))

  for transaction in block.transactions[1:]:
    ordinals = []
    for input in transaction.inputs:
      ordinals.extend(input.ordinals)

    for output in transaction.outputs:
      output.ordinals = ordinals[:output.value]
      del ordinals[:output.value]

    coinbase_ordinals.extend(ordinals)

  for output in block.transaction[0].outputs:
    output.ordinals = coinbase_ordinals[:output.value]
    del coinbase_ordinals[:output.value]
```

---

# What are ordinals good for?

If you want a token, you can just pick and ordinal to represent your token, and
use the location of the ordinal to represent ownership.

The person who controls the private key that corresponds to the public key of
the UTXO that contains the ordinal is the current owner of the token.

---

# What else are ordinals good for?

- Aesthetics!
- Supporting the fee market!

---

# Wacky aside: Ordinal traits

- 😏 Rare ordinals
- 🤤 Epic ordinals
- 🥵 Legendary ordinals
- Bounties: https://ordinals.com/bounties/

---

# Ordinal Index

- [big](http://api.ordinals.com:8000/list/e11d223685e110c5df93d7ae57f63c535ac59d1d65c16de779f23a9166229c7e:0)
- [small](http://api.ordinals.com:8000/list/81bb70199e0c2cf6a32ee0b8079085eb590c311f6e91bb51c14b85846593a76e:1)
- [spent](http://api.ordinals.com:8000/list/b40375d8e4f50728c18ed292c2e40ed616797592a2f5587c9f72a23a55973f9e:0)

---

# What are ordinals not good for?

- Not having to make weird multi-step transactions to avoid hitting the dust
  limit
- Being efficient with block space
- Very high divisibility
- Small databases

---

# Ordinal NFTs

1. Hash: (ordinal || content hash || public key)
2. Sign
3. Append signature, data, and then bech32 encode
4. Et voilà: `nft1qqz3psl8mvjm9t573n29l8q0phklcdhg65392pv0gpc79tydeujltn5g2h4fsg...`

---

# Ordinal NFTs

- No on-chain transaction to mint
- Store the NFT wherever
- Anyone who has access to the NFT will know the secret, hidden meaning of the
  ordinal.

---

# Ordinal NFT Upgrades: Issued by ordinal owner

- Currently, anyone can assign an NFT to an ordinal.
- This strikes some people as weird, like digital graffiti.
- In the future, may want to give some special status, like "self-issued" if the NFT is signed with a public key that held the ordinal.

---

# Ordinal NFT Upgrades: Timestamping

- Age is of great interest to NFT collectors
- Including a timestamp in an NFT is an easy way to prove the age of an NFT

---

# Ordinal NFT Upgrades: Publicity

- Timestamps prove that an NFT was created before a particular time
- Timestamps *don't* prove that there wasn't anything that was created earlier
- It might be desirable to add a public issuance method, i.e. one where the chain can be scanned for such issuences, and a total ordering to be established over them
- Very reluctant to do this, since this kind of embedding of public data in the Bitcoin blockchain is not very aesthetically pleasing or culturally acceptable to bitcoiners
