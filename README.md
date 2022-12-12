`ord`
=====

`ord` is an index, block explorer, and command-line wallet.

Ordinals are serial numbers for satoshis, assigned in the order in which they
are mined, and preserved across transactions.

See [the BIP](bip.mediawiki) for a comprehensive description of the assignment
and transfer algorithm.

See [the project board](https://github.com/users/casey/projects/3/) for
currently prioritized issues.

See [milestones](https://github.com/casey/ord/milestones) to get a sense of
where the project is and where it's going.

Join [the Discord server](https://discord.gg/87cjuz4FYg) to chat with fellow
ordinal degenerates.

Contributing
------------

### Suggested Steps

1. Find an issue you want to work on.
2. Figure out what would be a good first step towards resolving the issue. This
   could be in the form of code, research, a proposal, or suggesting that it be
   closed, if it's out of date or not a good idea in the first place.
3. Comment on the issue with an outline of your suggested first step, and
   asking for feedback. Of course, you can dive in and start writing code or
   tests immediately, but this avoids potentially wasted effort, if the issue
   is out of date, not clearly specified, blocked on something else, or
   otherwise not ready to implement.
4. If the issue requires a code change or bugfix, open a draft PR with tests,
   and ask for feedback. This makes sure that everyone is on the same page
   about what needs to be done, or what the first step in solving the issue
   should be. Also, since tests are required, writing the tests first makes it
   easy to confirm that the change can be tested easily.
5. Mash the keyboard randomly until the tests pass, and refactor until the code
   is ready to submit.
6. Mark the PR as ready to review.
7. Revise the PR as needed.
8. And finally, mergies!

### Advice

#### Start small

Small changes will allow you to make an impact
quickly, and if you take the wrong tack, you won't have wasted much time.

Ideas for small issues:
- Add a new test or test case that increases test coverage
- Add or improve documentation
- Find an issue that needs more research, and do that research and summarize it
  in a comment
- Find an out-of-date issue and comment that it can be closed
- Find an issue that shouldn't be done, and provide constrictive feedback
  detailing why you think that is the case

#### Merge early and often

Break up large tasks into multiple smaller steps that individually make
progress. If there's a bug, you can open a PR that adds a failing ignored test.
This can be merged, and the next step can be to fix the bug and unignore the
test. Do research or testing, and report on your results. Break a feature into
small sub-features, and implement them one at a time.

Figuring out how to break down a larger PR into smaller PRs where each can be
merged is a art form well-worth practicing. The hard part is that each PR must
itself be an improvement.

I strive to follow this advice myself, and am always better off when I do.

Small changes are fast to write, review, and merge, which is much more fun than
laboring over a single giant PR that takes forever to write, review, and merge.
Small changes don't take much time, so if you need to stop working on a small
change, you won't have wasted much time as compared to a larger change that
represents many hours of work. Getting a PR in quickly improves the project a
little bit immediately, instead of having to wait a long time for larger
improvement. Small changes are less likely to accumulate merge conflict. As the
Athenians said: *The fast commit what they will, the slow merge what they
must.*

#### Get help

If you're stuck for more than 15 minutes, ask for help, like a Rust Discord,
Stack Exchange, or in a project issue or discussion.

#### Practice hypothesis-driven debugging

Formulate a hypothesis as to what is causing the problem. Figure out how to
test that hypothesis. Perform that tests. If it works, great, you fixed the
issue or now you know how to fix the issue. If not, repeat with a new
hypothesis.

#### Pay attention to error messages

Read all error messages, and don't tolerate warnings.

### Copyright

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

### Minimum System Requirements

Building the `ord` index is memory-intensive. An `ord` instance running on a
reasonably powerful system with 64 GiB of RAM and a fully-synced,
locally-hosted Bitcoin Core node can build the index in under 24 hours.

An `ord` instance running on a system with less than 64 GiB of RAM will take
much longer to build the index, with some users reporting sync times of two
weeks or more.

A this point in development, the format of the `ord` index is changing rapidly,
so be prepared to delete and re-sync the index.

### Logging

`ord` uses [env_logger](https://docs.rs/env_logger/latest/env_logger/). Set the `RUST_LOG` environment variable in order to turn on logging. For example, run the server and show `info`-level log messages and above:

```
$ RUST_LOG=info cargo run server
```

Reorgs
------

Currently, reorganizations are detected but not handled. After reorgs happen,
you'll need to delete `index.redb` and start over.
