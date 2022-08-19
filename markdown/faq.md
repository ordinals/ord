Ordinal FAQ
===========

How many ordinals will there be?
--------------------------------

THERE WILL ONLY EVER BE 2,099,999,997,690,000 ORDINALS

What are ordinals?
------------------

Ordinals are serial numbers for satoshis, like this: 804766073970493. Every
satoshi, which is ¹⁄₁₀₀₀₀₀₀₀₀ of a bitcoin, has an ordinal, and every ordinal
has a satoshi. When a satoshi changes hands, so does its ordinal. There are a
lot of satoshis, and so there are a lot of ordinals. Two quadrillion
ninety-nine trillion nine hundred ninety-nine billion nine hundred ninety-seven
million six hundred ninety thousand, to be precise.

What are ordinals not?
----------------------

Ordinals are not a side chain, a separate token, something that requires any
changes to Bitcoin, or complicated.

What are ordinals for?
----------------------

Collecting, trading, and scheming. Ordinals could be used as the basis of
Bitcoin-native NFTs, using a scheme to assign NFTs to ordinals, and then use
those ordinals to track ownership of the NFT. But only could, nobody's written
the code for that yet.

How do ordinals work?
---------------------

Ordinals are assigned to satoshis in the ordinal in which they are mined. The
first satoshi in the first block is ordinal 0, the second is ordinal 1, and the
last satoshi of the firt block is ordinal 4,999,999,999.

Ordinals live in UTXOs, but transactions destroy spent inputs and create new
outputs, so we need an algorithm to determine how ordinals hop from the inputs
to the outputs of a transaction. Fortunately, that algorithm is very simple.

Ordinals transfer in first-in-first-out order. Think of the inputs to a
transaction as being a list of ordinals, and the outputs as a list of slots,
waiting to receive an ordinal. To assign ordinals to slots, go through each
ordinal in the inputs in order, and assign each to the first available slot in
the outputs.

Let's imagine a transaction with three inputs and two outputs. The inputs are
on the left of the arrow and the outputs are on the right, all labeled with
their values:

    [2] [1] [3] → [4] [2]

Now let's label the same transaction with the ordinals that each input
contains, and question marks for each output ordinal. Ordinals are long
numbers, so let's use letters to represent them:

    [a b] [c] [d e f] → [? ? ? ?] [? ?]

To figure out which ordinal goes to which output, go through the input ordinals
in order and assign each to a question mark:

    [a b] [c] [d e f] → [a b c d] [e f]

What about fees, you might ask? Good question! Let's imagine the same
transaction, this time with a two satoshi fee. Transactions with fees send more
satoshis in the inputs than are received by the outputs, so to make our
transaction into one that pays fees, we'll remove the second output:

    [2] [1] [3] → [4]

The ordinals, <var>e</var> and <var>f</var> now have nowhere to go in the outputs:

    [a b] [c] [d e f] → [a b c d]

So they go to the miner who mined the block as fees. [The
BIP](https://github.com/casey/ord/blob/master/bip.mediawiki) has the details,
but in short, fees paid by transactions are treated as extra inputs to the
coinbase transaction, and are ordered how their corresponding transactions are
ordered in the block. The coinbase transaction of the block might look like
this:

    [SUBSIDY] [e f] → [SUBSIDY e f]

Where can I find the nitty-gritty details?
------------------------------------------

[The BIP!](https://github.com/casey/ord/blob/master/bip.mediawiki)
