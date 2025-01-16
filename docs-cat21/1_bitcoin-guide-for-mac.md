# Bitcoin / Ord Guide for Mac

Main source for this: [Mac beginner's guide by ETS](https://discord.com/channels/987504378242007100/1078719003624747108) – **This whole guide is a blatant copy & paste, I deserve zero credits!**

This guide mostly assumes a primary SSD and an external SSD, formatted as APFS. If you want to follow the commands verbatim, name your external SSD volume as **ord-dev**.  You'll still need  ~100 GB free on your internal disk, as only the bitcoin blocks are moved to the external SSD. This configuration allows for moving the bulk of the storage needs to the external but leaving enough in the default locations to make commands easier.

⚠️ This guide won't work well if your primary or external storage are on a spinning disk. SSD only! **Like really, you will regret your wasted time otherwise!** Also make sure that you have a lot of memory – otherwise ord will never finish. Even machines with 32 GB+ are struggling sometimes these days. Go for 64 GB!

## 1️⃣ INSTALL HOMEBREW

Install Homebrew from http://brew.sh/
During installation you may be prompted for a password. This is your main Mac password and you won't see any letters as you type it. After typing it, press enter to continue.

⚠️ When installation completes, follow the steps listed in "Next steps" to add Homebrew to your path.

## 2️⃣ INSTALL BITCOIN

After Homebrew installation and "Next steps", run this  from terminal:

```sh
brew install bitcoin
```

## 3️⃣ CREATE  BLOCKS DIR

⚠️Skip this step if no external SSD.

After the bitcoin installation, create your bitcoin blocks directory on your external SSD (most Macs don't have enough storage on their internal SSDs. 1 TB free for comfort.) Name the folder `bitcoin`. If your external SSD volume name has a space, rename that, too. No spaces! In this guide, the external SSD has the name `ord-dev`!

## 4️⃣ CONFIGURE BITCOIN.CONF

Run these commands in Terminal:

```sh
mkdir ~/Library/Application\ support/Bitcoin

echo blocksdir=/Volumes/ord-dev/Bitcoin > ~/Library/Application\ Support/Bitcoin/bitcoin.conf
echo txindex=1 >> ~/Library/Application\ Support/Bitcoin/bitcoin.conf
echo server=1 >> ~/Library/Application\ Support/Bitcoin/bitcoin.conf
echo mempoolfullrbf=1 >> ~/Library/Application\ Support/Bitcoin/bitcoin.conf

echo blocksdir=/Volumes/ord-dev/Bitcoin\\ntxindex=1\\nserver=1\\n > ~/Library/Application\ support/Bitcoin/bitcoin.conf
```

(where the `/ord-dev/Bitcoin` part is the actual path to the SSD and folder you created above. CAREFUL MODIFYING THIS COMMAND, EVERY SPACE AND SLASH AND ~ IS NEEDED!)

⚠️ If you don't have an external drive, then skip the `blocksdir` line!

* `txindex=1` If you want to be able to access any transaction with commands like gettransaction , you need to configure Bitcoin Core to build a complete transaction index, which can be achieved with the txindex option.
* `server=1` tells bitcoin to accept JSON-RPC commands, so you can query it

Confirm all settings with:

```sh
cat ~/Library/Application\ Support/Bitcoin/bitcoin.conf
```


## 5️⃣ START/STOP BITCOIN SERVICE

In terminal:

```sh
brew services start bitcoin
```

and this if you want to stop it again (not yet)

```sh
brew services stop bitcoin
```

## 6️⃣ CONFIRM SETTINGS

In the `debug.log` file located in `~/Library/Application\ Support/Bitcoin/`, look for lines similar to these:

```log
2023-11-15T13:16:47Z Default data directory /Users/username/Library/Application Support/Bitcoin
2023-11-15T13:16:47Z Using data directory /Users/username/Library/Application Support/Bitcoin
2023-11-15T13:16:47Z Config file: /Users/username/Library/Application Support/Bitcoin/bitcoin.conf
2023-11-15T13:16:47Z Config file arg: blocksdir="/Volumes/ord-dev/Bitcoin"
2023-11-15T13:16:47Z Config file arg: server="1"
2023-11-15T13:16:47Z Config file arg: txindex="1"
2023-11-15T13:16:47Z Generated RPC authentication cookie /Users/username/Library/Application Support/Bitcoin/.cookie
```

⚠️ An easy way to monitor the file in Terminal is to use this command:

```sh
tail -F -n 10000 ~/Library/Application\ Support/Bitcoin/debug.log
```

These entries in the output confirm that the blocks dir and the configuration entries have been recognized and have taken effect. Pay attention to the cookie line. You'll need that later, so make sure you have it!

## 7️⃣ INITIAL BLOCK DOWNLOAD (aka BLOCKCHAIN SYNC)

The initial block download will generally take 1+ days, depending on factors like CPU/disk/network speed. As it progresses, you'll see lines like this in the debug.log file:

```sh
2023-02-22T15:18:23Z UpdateTip: new best=00000000000000000000e2319131e7e41d3b93e8b9086fc427f2ee9383aa2686 height=777656 version=0x20004000 log2_work=94.017345 tx=807469638 date='2023-02-21T14:40:00Z' progress=0.999679 cache=2.5MiB(18835txo)
```

The `progress=` entry tells you how far you are along. It will reach 1.000000 at completion of the initial block download and then you can leave bitcoin running as a service and proceed to the ord installation and configuration.

## 8️⃣ EXTRA: FINAL CONFIRMATION OF READINESS

Run this in terminal:

`bitcoin-cli getindexinfo`

You should see a response like this:

```json
{
  "txindex": {
    "synced": true,
    "best_block_height": 839505
}
```

If it shows `synced:false` or the block height is not current, wait longer and run the command again. If it shows `synced:true` and current block height, proceed to ord installation!

## 9️⃣ EXTRA: Setting Up Bitcoin Core to Accept Remote Procedure Calls (RPC) from an External Host

Here are some tips for getting the bitcoin core node up and running and accepting RPC connections from external hosts.
This is not an efficient setup in combination with `ord` (it should be much faster when everything runs on the same machine), but useful for other development tasks.

`~/Library/Application\ Support/Bitcoin/bitcoin.conf` (mac).
`/bitcoin/.bitcoin/bitcoin.conf` (linux)

```ini
server=1
rpcbind=0.0.0.0
rpcallowip=<your_network_range>
rpcport=8332
rpcauth=<username>:<hashed_password>
txindex=1
mempoolfullrbf=1
```

#### server=1

This tells tells Bitcoin to accept JSON-RPC commands.

#### rpcbind=0.0.0.0

This will tell Bitcoin to listen an every available network interface. If you would like to only accept connections on a specific network interface, replace `0.0.0.0` with the IP of your desired network interface.

#### rpcallowip=<your_network_range>

With `rpcallowip` you can either specify an IP range in CIDR format, or specify the rpcallowip option multiple times with distinct IP addresses that you want to allow connections from.

Note: Wildcards are no longer supported. You need to use subnets now.
To allow everything that would be `0.0.0.0/0` (ipv4) or  **`::/0`** (ipv6).
Of course, this is not save to expose to untrusted networks such as the public internet!

#### rpcport=8332

This sets the port that Bitcoin will listen on for RPC connections. Port 8332 is the default.

#### rpcauth=<username>:<hashed_password>

This configuration sets a username and password (hashed) to be used for authenticating RPC requests.
[Bitcoin Core RPC Auth Config Generator](https://jlopp.github.io/bitcoin-core-rpc-auth-generator/) is a useful tool for generating the hashed passwords.

#### txindex=1

With `txindex=1` Bitcoin Core maintains an index of all transactions that have ever happened, which you can query using the remote procedure call (RPC) method `getrawtransaction ` or the RESTful API call  `get-tx `.

Next, restart the bitcoind service!

Then, on the machine that you’ll be making RPC requests from, make a test request using `curl`:

```sh
curl -f --user <username> --data-binary \
    '{"method":"getblockhash","params":[0],"id":1}' \
    -H 'content-type: text/plain;' http://<hostname>:<port>/
```

In the above request, replace the following values:

* **`<username>`** – Replace this with the value you specified for rpcuser in your bitcoin.conf file.
* **`<hostname>:<port>`** - Replace **`<hostname>`** with the IP address or hostname of your Bitcoin server and **`<port>`** with the value you specified for `rpcport` in your `bitcoin.conf` file Hint: execute `ifconfig` to figure out your IP address.

If everything is set up correctly, after running the `curl` command, you should be prompted for your RPC password and Bitcoin will respond with a valid result.
