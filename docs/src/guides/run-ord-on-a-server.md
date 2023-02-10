Run ord on a server
===================

We host a public instance of the `ord` server over at
[ordinals.com](https://ordinals.com), and you can easily host your own on a good
enough server. This guide walks through setting up `ord` on a
[Hetzner](https://www.hetzner.com/) box using a provided deploy script.

In particular:

1. Navigating [Hetzner](https://www.hetzner.com/) and choosing a server
2. Setting up the [Hetzner](https://www.hetzner.com/) box
3. Deploying `ord`

Getting Help
------------

If you get stuck, try asking for help on the
[Ordinals Discord Server](https://discord.com/invite/87cjuz4FYg), or checking
GitHub for relevant [issues](https://github.com/casey/ord/issues) and
[discussions](https://github.com/casey/ord/discussions).

Choosing the right server
------------------------

Any server with ~1TB of disk space should be enough to run `ord`. Hetzner offers
a variety of solutions, but a simple ~$50/month dedicated root
server with 2 x 512GB disks will be enough to host `ord`. You can find more
information on the [dedicated root server page](https://www.hetzner.com/de/dedicated-rootserver).

Setting up the server
---------------------

By default you'll be launched in the Hetzner Rescue System, which is a Debian
based Linux environment that gives you admin access to your server. The
environment comes with a handy `installimage` script that lets you easily
install various Linux distributions. You can find more information on the
official [`installimage` documentation page](https://docs.hetzner.com/robot/dedicated-server/operating-systems/installimage).
This guide assumes you install Ubuntu 22.04.1 LTS base image.

Deploying `ord`
---------------

We provide a deploy script in the main repository that handles
firewall configuration and running `bitcoind` alongside `ord`.

Once you have your server setup, clone the `ord` repository from GitHub:

```
git clone https://github.com/casey/ord.git
```

Navigate into the root:

```
cd ord
```

Install requirements on the remote server, note you'll have to replace
`{{domain}}` with the IP address of your server:

```
ssh root@{{domain}} "mkdir -p deploy \
    && apt-get update --yes \
    && apt-get upgrade --yes \
    && apt-get install --yes git rsync"
```

Transfer the `checkout` script over onto the remote machine, once again
replacing `{{domain}}` with the IP address of your server:

```
rsync -avz deploy/checkout root@{{domain}}:deploy/checkout
```

Run the `checkout` script on the remote machine, replace `{{branch}}` and
`{{chain}}` with the git repository branch you'd like to deploy and on which
Bitcoin chain respectively. The `{{chain}}` argument should be a valid git
branch and the `{{chain}}` argument should be one of `mainnet | main, signet, regtest
or testnet | test`.

```
ssh root@{{domain}} 'cd deploy && ./checkout {{branch}} {{chain}} {{domain}}'
```

Alternatively, if you have `just` installed, you can run the following in the
project root with the appropriate configuration:

```
just deploy {{domain}} {{branch}} {{chain}}
```
