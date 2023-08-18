铭文指引
=========================

单个 聪 可以刻有任意内容，创建可以保存在比特币钱包中并使用比特币交易传输的比特币原生数字人工制品。 
铭文与比特币本身一样持久、不变、安全和去中心化。 

使用铭文需要一个比特币完整节点，让您了解比特币区块链的当前状态，以及一个可以创建铭文并在构建交易
以将铭文发送到另一个钱包时执行 聪 控制的钱包。 

Bitcoin Core 提供比特币全节点和钱包。 但是，Bitcoin Core 钱包不能创建铭文，不执行 聪 控制。 

这需要[`ord`](https://github.com/ordinals/ord)，序数实用程序。 `ord` 没有自己的钱包，
因此  `ord wallet`子命令与 Bitcoin Core 钱包交互。

本指南涵盖：

1. 安装 Bitcoin Core
2. 同步比特币区块链
3. 创建 Bitcoin Core 钱包
4. 使用 `ord wallet receive`收取聪
5. 使用`ord wallet inscribe`创建铭文
6. 使用 `ord wallet send`发送铭文
7. 使用`ord wallet receive`收取铭文

寻求帮助
------------

如果你遇到困难，可以在[Ordinals Discord Server](https://discord.com/invite/87cjuz4FYg), 或者检查Github上的相关内容[问题](https://github.com/ordinals/ord/issues) 和[讨论](https://github.com/ordinals/ord/discussions).

安装Bitcoin Core
-----------------------

Bitcoin Core 可以在 [bitcoincore.org](https://bitcoincore.org/) 上的[下载页面](https://bitcoincore.org/en/download/).

制作铭文需要Bitcoin Core 24 或者更新版本。

本指南不包括如何详细安装 Bitcoin Core；当你成功安装Bitcoin Core以后，你应该可以在命令行使用 `bitcoind -version`命令。

配置 Bitcoin Core
------------------------

`ord` 需要Bitcoin Core 的交易索引

配置你的Bitcoin Core阶段去维护一个交易索引，需要在`bitcoin.conf`里面添加:

```
txindex=1
```

或者, 运行 `bitcoind` 和 `-txindex`:

```
bitcoind -txindex
```

比特币区块同步
------------------------------

区块同步，运行：

```
bitcoind -txindex
```

…直到运行 `getblockcount`:

```
bitcoin-cli getblockcount
```

像区块链浏览器[the mempool.space block explorer](https://mempool.space/)一样对区块进行记述. `ord`同`bitcoind`进行交互, 所以你在使用`ord`时候需要让`bitcoind` 在后台运行。

安装 `ord`
----------------

`ord` 程序使用Rust语言写成，可以从[源码](https://github.com/ordinals/ord)安装. 预制文件可以从[版本发布页](https://github.com/ordinals/ord/releases)下载。

你也可以在命令行中使用下面命令来安装最新的文件：

```sh
curl --proto '=https' --tlsv1.2 -fsLS https://ordinals.com/install.sh | bash -s
```

当 `ord` 成功安装以后,你可以运行 :

```
ord --version
```

这会返回 `ord`的版本信息.

创建一个Bitcoin Core钱包
------------------------------

`ord` 使用Bitcoin Core来管理私钥，签署交易以及向比特币网络广播交易。

创建一个名为`ord` 的Bitcoin Core 钱包，运行:

```
ord wallet create
```

接收聪
--------------

铭文是在单个聪上制作的，使用聪来支付费用的普通比特币交易，因此你的钱包将需要一些 聪（比特币）。

为你的 `ord` 钱包创建一个新地址，运行:

```
ord wallet receive
```

像上面地址发送一些资金。你可以使用以下命令看到交易情况：

```
ord wallet transactions
```

一旦交易确认，你应该可以使用 `ord wallet outputs`看到交易的输出；


创建铭文内容
----------------------------

聪上可以刻录任何类型的内容，但`ord`钱包只支持`ord`区块浏览器可以显示的内容类型。

另外，铭文是包含在交易中的，所以内容越大，铭文交易需要支付的费用就越高。 

铭文内容包含在交易见证中，获得见证折扣。要计算写入交易将支付的大概费用，请将内容大小除以四，然后乘以费率。 

铭文交易必须少于 400,000 个权重计量单位，否则不会被 Bitcoin Core 中继。一个字节的铭文内容需要
一个权重计量单位。 由于铭文交易不只是铭文内容，铭文内容限制在400,000权重计量单位以内。 
390,000 个权重计量单位应该是安全的。

创建铭文
---------------------

以`FILE`的内容创建一个铭文，需要运行:

```
ord wallet inscribe --fee-rate FEE_RATE FILE
```

Ord会输出两个交易ID，一个是commit交易，一个是reveal交易，还有铭文ID。 
铭文 ID 的格式为`TXIDiN`，其中`TXID` 是揭示交易的交易 ID，`N` 是揭示交易中铭文的索引。

Commit交易提交到包含铭文内容的 tapscript，reveal交易则从该 tapscript 中花费，
显示链上的内容并将它们铭刻在reveal交易的第一个输出的第一个 sat 上。

在等待reveal交易被记录的同时，你可以使用[the mempool.space block explorer](https://mempool.space/)来
检查交易的状态。

一旦reveal交易完成记账，你可以使用以下命令查询铭文ID：

```
ord wallet inscriptions
```

你可以在 [the ordinals explorer](https://ordinals.com/) 上用一下格式访问铭文
`ordinals.com/inscription/INSCRIPTION_ID`.

发送铭文
--------------------

铭文接收方使用一下命令生成地址

```
ord wallet receive
```

使用命令格式发送铭文：

```
ord wallet send --fee-rate <FEE_RATE> <ADDRESS> <INSCRIPTION_ID>
```

检查交易情况：

```
ord wallet transactions
```

一旦交易确认，接收方可以使用一下命令查看接收到的铭文

```
ord wallet inscriptions
```

接收铭文
----------------------

使用命令生成钱包地址：

```
ord wallet receive
```

发送方使用命令发送铭文

```
ord wallet send ADDRESS INSCRIPTION_ID
```

检查交易情况

```
ord wallet transactions
```

一旦交易确认，你可以使用一下命令查看接收到的铭文

```
ord wallet inscriptions
```
