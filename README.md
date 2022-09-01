`ord`
=====

`ord` is an ordinal index, block explorer, and command-line ordinal wallet.

Ordinals are serial numbers for satoshis, assigned in the order in which they
are mined, and preserved across transactions.

See [the BIP](bip.mediawiki) for a comprehensive description of the assignment
and transfer algorithm.

See [the project board](https://github.com/casey/ord/projects/1) for currently
prioritized issues.

Join [the Discord server](https://discord.gg/87cjuz4FYg) to chat with fellow
ordinal degenerates.

Contributing
------------

Find an issue you like with the [good first
issue](https://github.com/casey/ord/labels/good%20first%20issue) label. Before
you start working, comment on the issue saying you're interested in working on
it. The issue may already be implemented, out of date, or not fully fleshed
out.

`ord` is extensively tested, and all PRs with new functionality or bug fixes
require tests. Before starting to write code, open a draft PR with failing
tests that demonstrate the functionality to be written, or the bug to be fixed.
This allows the maintainers to make sure that everyone is on the same page, and
that there's a good strategy to test the PR. Once that's done, you can start
writing the actual code.

`ord` is licensed under the CC0, a no-strings-attached public domain dedication
and fallback license. Your changes must be licensed under the CC0, without any
additional terms or conditions

Running `ord`
-------------

`ord` requires a synced `bitcoind` node with `-txindex` to build the index of
ordinal locations. `ord` communicates with `bitcoind` via RPC.

If `bitcoind` is run locally by the same user, without additional
configuration, `ord` should find it automatically by reading the `.cookie` file
from `bitcoind`'s datadir, and connecting using the default RPC port.

If `bitcoind` is not on mainnet, is not run by the same user, has a non-default
datadir, or a non-default port, you'll need to pass additional flags to `ord`.
See `ord --help` for details.

Index
-----

The mainnet ordinal index is currently 50 GiB, and will increase in size in the
future, both as the Bitcoin blockchain grows, and as additional tables are
added to the index.

The signet ordinal index is much smaller, clocking in at 1.3 GiB.

Reorgs
------

Currently, reorganizations are detected but not handled. After reorgs happen,
you'll need to delete `index.redb` and start over.
