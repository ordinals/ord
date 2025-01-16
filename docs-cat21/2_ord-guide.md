## 1️⃣ INSTALL ORD

Run this command in terminal:

```sh
brew install ord
```

Confirm success by typing this in terminal:

```sh
ord --version
```

You should get a response like this (or a higher version):

```sh
ord 0.20.0
```

## 2️⃣ CREATE ORD INDEX

⚠️ If you don't have an internal SSD (you have an internal spinning disk) this may not work well. Seek assistance to relocate your index file with the --index switch. Doing so will change all following ord commands in this guide, as well.

We will create a full-blown ord index with sats and runes on your internal SSD. Run this command:

```sh
ord --index-runes --index-sats --index ~/ordindex.redb  server
```

If all's well you'll see an indexing progress bar. It will start fast and slow down considerably. If it completes, great -- often it doesn't if your computer's resources are constrained. If it slows or appears to stop--WAIT LONGER. A trick that seems to work if it completely bombs out at the end or stops for many hours, is to press ctrl-c (only once!) when it's a few blocks from completing, then give it time to exit gracefully, reboot and run it again. Repeat until successful.

#### --index-sats
Track location of all satoshis. Note: This tracks the current location of the sat, not the history. Spent outputs won't show sat-ranges.

<!--
#### --index-spent-sats

I'm not sure it has ever been successfully built by anyone.
It essentially tracks the history for every single sat ever spent, so you can see who has held it since it was created.
The DB it makes is enormous, like multiple TB.
-->

#### --index-addresses

Track unspent output addresses. This allows you to view the contents of an address within ord.

## JUST DOWNLOAD AN ORD INDEX

Tired of waiting? Ok, there are Ordinals Index files pre-built by Greg.
These are the pre-built `index.redb` files, ready for download:

https://ordstuff.info/

Thanks Greg! 🙏

Yeah, that's just the .torrent file. You need to plug that into a torrent app to download the multi-gigabyte .redb.gz file.

Two possible clients for Mac:
* [Transmission](https://transmissionbt.com/)
* [qbittorrent](https://www.qbittorrent.org/)
