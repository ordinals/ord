Reindexing
==========

Sometimes the `ord` database must be reindexed, which means deleting the
database and restarting the indexing process with either `ord index update` or
`ord server`. Reasons to reindex are:

1. A new major release of ord, which changes the database scheme
2. The database got corrupted somehow

The database `ord` uses is called [redb](https://github.com/cberner/redb),
so we give the index the default file name `index.redb`. By default we store this
file in different locations depending on your operating system.

|Platform | Value                                            | Example                                      |
| ------- | ------------------------------------------------ | -------------------------------------------- |
| Linux   | `$XDG_DATA_HOME`/ord or `$HOME`/.local/share/ord | /home/alice/.local/share/ord                 |
| macOS   | `$HOME`/Library/Application Support/ord          | /Users/Alice/Library/Application Support/ord |
| Windows | `{FOLDERID_RoamingAppData}`\ord                  | C:\Users\Alice\AppData\Roaming\ord           |

So to delete the database and reindex on MacOS you would have to run the following
commands in the terminal:

```bash
rm ~/Library/Application Support/ord/index.redb
ord index update
```

You can of course also set the location of the data directory yourself with `ord
--data-dir <DIR> index update` or give it a specific filename and path with `ord
--index <FILENAME> index update`.
