# Bitcoin Atoms

Discover the current whereabouts of Bitcoin Atoms.

Atoms are created, numbered, and transferred according to a few simple rules:

- Every new block creates an atom as an implicit input to the coinbase
  transaction, similar to the block subsidy.

- Atoms are numbered according to the height of the block in which they were
  created. The atom created in the genesis block is atom 0, the next atom 1,
  and so forth.

- Atoms are carried by coins.

- Atoms are transferred when the coins carrying them are used as inputs to a
  transaction. In the absence of a specially crafted transaction, they move
  deterministically to output coins proportionately to the value of each coin.
  A specially crafted `apportion` transaction allows the owner of atoms to
  apportion them to output coins at will. However, the precise form of
  `apportion` transactions has been lost to history. Hopefully it will soon be
  rediscovered.

- Atoms can be transmuted. Atoms start out devoid of character, aside from
  their number. However, they are Promethean. The owner of an atom may change
  its character with the `transmute` operation, associating a 256 bit number
  with it. This might be the SHA-256 hash of an image, thus creating an NFT.
  However, we all know that humans are chaotic and inventive. Powerful new uses
  for atoms may as yet be undiscovered, lurking now merely as phantoms in the
  heads of heretics. The precise mechanism of `transmute` has not been
  discovered. Speculation on this matter is welcome.

  Transmutation may be repeated, if the old form of an atom has become
  tiresome. Thus do scarce atoms seek their best and highest use.

Atoms have many nice properties:

- Fair: Atoms are not premined or minted arbitrarily. Atoms are awarded to
  miners, just like bitcoin itself.

- Decentralized: Since atoms have been created in every block, including the
  genesis block, and without intention, they are distributed among existing
  coin holders proportionally to their holdings.

- Scarce: As of midnight, December 12th, 2021, there are 713,802 atoms, and
  only 144 more will be created every day. This is quite a bit more than a few,
  but quite a bit less than the untold billions of digital assets being churned
  out of the vast VC-funded forges.

- Precedented: They are similar in spirit to [something that Satoshi included
  in Bitcoin, but removed before
  launch](https://github.com/JeremyRubin/satoshis-version/blob/2197a48a1432f567314ce6c6c4be9270518f882e/src/main.cpp#L1227).

- Security enhancing: Since new atoms go to miners, they contribute to
  Bitcoin's security budget, and the more use humans have of atoms, the greater
  that contribution. Since one atom is created per block, with no halving, they
  will continue to contribute to Bitcoin's security budget long after the block
  subsidy dwindles to zero.

- Voluntary: A coin holder can ignore atoms completely if they so choose, aside
  from seeing a few inscrutable `transmute` and `apportion` transactions on
  chain.

- Natural: Atoms are a natural extension to Bitcoin, mimicing Bitcoin's rules
  and incentives.
