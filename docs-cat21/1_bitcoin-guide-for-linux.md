# Bitcoin / Ord Guide for Linux

Main source for this: [Homebrew appreciation thread for Bitcoin and ord on Mac and Linux by ETS](https://discord.com/channels/987504378242007100/1078719003624747108) – **This whole guide is a blatant copy & paste, I deserve zero credits!**

This guide assumes a 2TB primary SSD and 64GB of Ram. Even machines with 32GB+ sometimes struggle these days. Go for 64GB!
I did this installation on a "debian 12 (bookworm) - minimal" VPS.

## 0️⃣ Only if required: Installing Curl on Debian

First apply patches for your system:

```sh
sudo apt update && sudo apt upgrade
```

Install `curl` (necessary) and other build tools:

```sh
apt install build-essential procps curl file git

curl --version
```


## 1️⃣ INSTALL HOMEBREW

Install Homebrew from http://brew.sh/, the webpage should tell you to execute:

```
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

If you get the error: **"Don't run this as root!"** then you are logged in as "root". The issue here is that Homebrew does not recommend or allow installation as the root user for security reasons. Instead, it should be installed under a regular user account with sudo privileges.

Here's how you can resolve the issue:

Create a new user (if you don't already have one) and give it sudo privileges. Here's how to do that for a user called `ord-dev`

```
adduser ord-dev
```

Follow the prompt to set up a password.
Add the user to the sudo group:

```sh
usermod -aG sudo ord-dev
```

Switch to the new user:

```sh
su - ord-dev
```

Now try again:

```
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

⚠️ When installation completes, follow the steps listed in "Next steps" to add Homebrew to your path.

## 2️⃣ INSTALL BITCOIN

After Homebrew installation and "Next steps", run this  from terminal:

```sh
brew install bitcoin
```

## 3️⃣ CONFIGURE BITCOIN.CONF

Run these commands in Terminal:

```sh
mkdir ~/.bitcoin/bitcoin.conf

echo txindex=1 >> ~/.bitcoin/bitcoin.conf
echo server=1 >> ~/.bitcoin/bitcoin.conf
echo mempoolfullrbf=1 >> ~/.bitcoin/bitcoin.conf
```

* `txindex=1` If you want to be able to access any transaction with commands like gettransaction , you need to configure Bitcoin Core to build a complete transaction index, which can be achieved with the txindex option.
* `server=1` tells bitcoin to accept JSON-RPC commands, so you can query it

Confirm all settings with:

```sh
cat ~/.bitcoin/bitcoin.conf
```

## 4️⃣ START/STOP BITCOIN SERVICE

In terminal:

```sh
brew services start bitcoin
```

and this if you want to stop it again (not yet)

```sh
brew services stop bitcoin
```

If this works you are lucky and can skip section 5.

## 5️⃣ TROUBLESHOOTING (for Debian)

How to fix this error:

```sh
Failed to connect to bus: No medium found
Error: Failure while executing; `/usr/bin/systemctl --user daemon-reload` exited with 1.
```

This error occurs because `systemctl` (used by brew services) requires `systemd` to be set up for user sessions, which is not always enabled by default in systems like Debian, especially in minimal or server installations. Switching to `systemctl ` for managing services like `bitcoin` and `ord` is a more sustainable and reliable solution on Debian. Homebrew works well for package management, but it struggles with service management on Linux, particularly in systems like Debian that don’t fully support `systemctl --user` by default.

```sh
sudo nano /etc/systemd/system/bitcoind.service
```

Add the following content:

```ini
[Unit]
Description=Bitcoin Daemon
After=network-online.target
Wants=network-online.target

[Service]
ExecStart=/home/linuxbrew/.linuxbrew/opt/bitcoin/bin/bitcoind
User=ord-dev
Restart=on-failure
Type=simple

[Install]
WantedBy=multi-user.target
```

Then execute the following commands to start the daemon:

```sh
sudo systemctl daemon-reload
sudo systemctl enable bitcoind
sudo systemctl start bitcoind
sudo systemctl status bitcoind
```

While we are here, this is a working the configution for `ord` (that was installed via homebrew):

```sh
sudo nano /etc/systemd/system/ord.service
```

Add the following content:

```ini
[Unit]
Description=Ord Daemon
After=network.target

[Service]
AmbientCapabilities=CAP_NET_BIND_SERVICE
Environment=RUST_BACKTRACE=1
Environment=RUST_LOG=info
ExecStart=/home/linuxbrew/.linuxbrew/bin/ord \
  --index-runes \
  --index-sats \
  --index-addresses \
  server \
  --http
User=ord-dev
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

`AmbientCapabilities=CAP_NET_BIND_SERVICE:` This grants the `ord` service the specific capability to bind to privileged ports (like port 80) without running as root. The default location of the index is: `~/.local/share/ord`.

Run the ord daemon permanently:

```sh
sudo systemctl daemon-reload
sudo systemctl enable ord
sudo systemctl start ord
sudo systemctl status ord
```

To view the full logs for your `ord` service managed by `systemd`, you can use `journalctl`, which collects logs for all systemd-managed services.
This will show the complete log history:

```sh
sudo journalctl -u ord
```

Or follow the logs in real-time (like `tail -f`):

```
sudo journalctl -u ord -f
```

To delete logs older than a specific time period, such as 1 week:
```
sudo journalctl --vacuum-time=1w
```

## 6️⃣ CONFIRM SETTINGS

In the `debug.log` file located in `~/.bitcoin`, look for lines similar to these:

```log
2024-09-24T19:28:09Z Default data directory /home/ord-dev/.bitcoin
2024-09-24T19:28:09Z Using data directory /home/ord-dev/.bitcoin
2024-09-24T19:28:09Z Config file: /home/ord-dev/.bitcoin/bitcoin.conf
2024-09-24T19:28:09Z Config file arg: server="1"
2024-09-24T19:28:09Z Config file arg: txindex="1"
2024-09-24T19:28:09Z Generated RPC authentication cookie /home/ord-dev/.bitcoin/.cookie
```

⚠️ An easy way to monitor the file in Terminal is to use this command:

```sh
tail -F -n 10000 ~/.bitcoin/debug.log
```

These entries in the output confirm that the configuration entries have been recognized and have taken effect. Pay attention to the cookie line. You'll need that later, so make sure you have it!

## 7️⃣ INITIAL BLOCK DOWNLOAD (aka BLOCKCHAIN SYNC)

The initial block download will generally take 1+ days, depending on factors like CPU/disk/network speed. As it progresses, you'll see lines like this in the debug.log file:

```sh
2023-02-22T15:18:23Z UpdateTip: new best=00000000000000000000e2319131e7e41d3b93e8b9086fc427f2ee9383aa2686 height=777656 version=0x20004000 log2_work=94.017345 tx=807469638 date='2023-02-21T14:40:00Z' progress=0.999679 cache=2.5MiB(18835txo)
```

The `progress=` entry tells you how far you are along. It will reach 1.000000 at completion of the initial block download and then you can leave bitcoin running as a service and proceed to the ord installation and configuration.

## 7️⃣ EXTRA: FINAL CONFIRMATION OF READINESS

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
