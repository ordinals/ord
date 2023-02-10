最近BTC社区推出了首个NFT项目[Ordinals](https://ordinals.com/),该项目非常的火爆，也体现出BTC社区强大的开发能力。鉴于其官方文档对基本原理的描述没有那么直观，并且实际操作Mint也非常复杂，因此本文将说明其核心原理和在BTC regtest本地网络上如何操作(主网操作方法移步:[全节点](https://docs.ordinals.com/guides/inscriptions.html)和[借助机器人不需要全节点](https://www.theblockbeats.info/news/34501?search=1))。

# 基本原理
Ordinal的技术分为两层：Ordinal number和铭刻(Inscriptions),前者用于为比特币的每一聪(sat)指定一个唯一序号，后者则是借助于Taproot来进行NFT的Mint。
> 注：虽然项目发起人很不屑将其称为NFT，而是称为Digital Artifacts,但是这里为便于理解将其简称为NFT。

## [Ordinal number理论](https://docs.ordinals.com/overview.html)
该理论提供了一种给聪(sat)指定一个唯一编号的机制，注意是每一个sat不是UTXO! 由于UTXO是不可再分的最小交易单元，因此sat只能存在于UTXO中，且UTXO包含了一定范围的sats，且只能在花费某一UTXO后产生新的输出中对sats编号进行拆分。sat的编号--Orinal number(一个sat和一个Ordinal number一一对应，为避免歧义，下文中sat和Ordinal number等价)主要根据以下两个规则确定:
1. 编号:每一个sat以他们被mint出来的顺序进行编号
2. 转移:按照先进先出规则，从交易的输入转移到输出

第一条规则相对简单,它决定了Orinal number**只能由挖矿奖励中的coinbase_tx生成**。例如，若第一个区块的挖矿奖励(不包含交易手续费)为50个btc(1 btc = 10^8 sat),则第一个区块会mint出[0;1;2;...;4,999,999,999]范围的sats；第二个区块奖励也为50 btc时，则第二个区块会mint出，[5,000,000,000;5,000,000,001;...;9,999,999,999]范围的sats。
这里比较难理解的部分在于，由于UTXO实际上包含很多个聪，那么这个UTXO中的每一个聪看起来都一样，怎么给他们排序呢？这个实际上是第二条规则决定的，举一个简单的例子吧:

假设中本聪的地址`addr_a`在创世区块mint了50个btc(1 btc = 10^8 sat),因此产生了[0;1;2;...4,999,999,999]这些sats(这些sats本身的相对顺序在转账交易后才能确定):
```
### inputs
### outputs
addr_a:[0 -> 4,999,999,999] sats
```
然后中本聪发起一笔交易，需要给addr_b转20个btc，那么:
```
### inputs
# 50 btc
addr_a: [0 -> 4,999,999,999] sats

### outputs
# 30 btc to addr_a, index=0
addr_a: [0 -> 2,999,999,999] sats
# 20 btc to addr_b, index=1
addr_b: [3,000,000,000 -> 4,999,999,999] sats
```
可以看出经过转账之后不同的UTXO集中包含的sat的顺序，是通过交易输出的index顺序按照先进先出被决定。

如果中本聪想进一步给addr_c转账10个btc，同时他还想保留第2,599,999,999个sat(因为这个sat上可能绑定了一个稀有NFT)，那么就需要精细地控制转账的顺序,确保这个sat还保存在自己钱包的UTXO集中:
```
### inputs
# 30 btc
addr_a: [0 -> 2,999,999,999] sats

### outputs
# 10 btc to addr_c, index=0
addr_c: [0 -> 999,999,999] sats
# 20 btc to addr_a, index=1
addr_a: [1,000,000,000 -> 2,999,999,999] sats
```

根据上述规则，每一个区块交易的UTXO中包含的sats可以通过以下[python代码](https://github.com/casey/ord/blob/master/bip.mediawiki#specification)表示:
```
# subsidy of block at given height
def subsidy(height):
  return 50 * 100_000_000 >> height // 210_000

# first ordinal of subsidy of block at given height
def first_ordinal(height):
  start = 0
  for height in range(height):
    start += subsidy(height)
  return start

# assign ordinals in given block
def assign_ordinals(block):
  first = first_ordinal(block.height)
  last = first + subsidy(block.height)
  coinbase_ordinals = list(range(first, last))

  for transaction in block.transactions[1:]:
    ordinals = []
    for input in transaction.inputs:
      ordinals.extend(input.ordinals)

    for output in transaction.outputs:
      output.ordinals = ordinals[:output.value]
      del ordinals[:output.value]

    coinbase_ordinals.extend(ordinals)

  for output in block.transaction[0].outputs:
    output.ordinals = coinbase_ordinals[:output.value]
    del coinbase_ordinals[:output.value]
```

这就是Ordinal NFT的核心技术支撑，非常的简洁但是却能衍生出很多好玩的东西! Ordinal Number甚至可以用来表示域名等。

## Ordinal Number表示方法和[稀缺性](https://docs.ordinals.com/overview.html#rarity)
Ordinal Number有很多种表示方式，这里主要解释其*度数表示法*(Degree Notation),其由4个字段构成:
```
A°B′C″D‴
│ │ │ ╰─ Index of sat in the block(每10分钟一个块)
│ │ ╰─── Index of block in difficulty adjustment period(每2016个块调整一次，~2周)
│ ╰───── Index of block in halving epoch(每210,000个块减半，~4年一次)
╰─────── Cycle, numbered starting from 0(减半和难度调整时间重合，~24年一次，最近一次大约在2032年)
```

虽然相同的度数会对应很多个sats，但是这种表示法有趣的地方在于，它根据比特币自身的周期性特征，人为地为sat创造了一种稀缺性：
- common: 所有不是区块mint出的第一个sat的sats
- uncommon: 该sat是某区块挖出的第一个sat(D==0)
- rare: 难度调整时挖出的第一个sat(C==0&&D==0)
- epic: 减半时挖出的第一个sat(B==0&&D==0)
- legendary: 发生Cycle轮换时挖出的第一个sat(B==C==D==0)
- mythic: 创世区块挖出的第一个sat(A==B==C==D==0)

比如，可在Ordinal 浏览器查看[`1°0′0″0‴`](https://ordinals.com/sat/1%C2%B00%E2%80%B20%E2%80%B30%E2%80%B4)这个传奇(legendary)sat,有趣的是作者根据UTXO包含的sats中稀有度级别，为每个UTXO标上了颜色。

还有一个表示法是每一个sat都有一个唯一的name，这个name随着挖出的区块越靠后越短，这又创造了一种稀缺性。

> 注: 虽然社区中很多人都fomo去mint punk之类的，花了高昂手续费也没mint到，但是我的观点倾向于和作者一致，最稀缺的是Ordinal Number而不是mint的内容中包含什么东西。仅为一家之言。

# [Inscriptions](https://docs.ordinals.com/inscriptions.html)
给Ordinal绑定相应的内容，如图片、文字等的过程叫做Inscriptions，我更喜欢叫mint。它主要基于2022年BTC Taproot升级之后引入的特性，可以在taproot脚本中存储任意数据(不超过4M)，并且同样可以利用隔离见证的1/4手续费打折，非常便宜且NFT完全上链。

由于taproot脚本具有很好的隐蔽性，只有在花费该解锁脚本时才能揭示脚本的内容，因此mint过程分为两步:
1. commit: 创建一个taproot输出(该输出会提交到一个包含mint内容的解锁脚本，此时script本身还未上链，只是生成了一条taproot script路径), 并提交该交易;
2. reveal: 花费上述交易生成的UTXO，由于此时必须将包含mint内容的脚本作为输入来解锁上述UTXO，因此就在链上揭示了mint的内容。

> 注: 这里涉及到很多Taproot和隔离见证的基础知识，限于篇幅就不在此展开，可以参考文末的参考资料。

通过`OP_FALSE OP_IF … OP_ENDIF`将mint的内容序列化到`tapscript`中，由于`OP_FALSE`这个判定条件为`False`，因此内容部分永远不会被执行，其丝毫不影响正常脚本的执行。比如包含`hello world`字符串在脚本中被序列化为:
```
OP_FALSE
OP_IF
  OP_PUSH "ord"
  OP_1
  OP_PUSH "text/plain;charset=utf-8"
  OP_0
  OP_PUSH "Hello, world!"
OP_ENDIF
```

# Regtest测试网Mint Ordinal NFT
主要包括下载bitcoin core和ord。
## bitcoin core下载安装
当前Ordinal NFT要求bitcoin core版本大于24,且使用的是bitcoind命令，因此mac的话注意不能下载`XX.dmg`文件，因为它不包含这个命令。

在[https://bitcoin.org/en/download]中找到自己适合自己操作系统的版本，然后下载。mac arm架构芯片(m1和m2)命令:
```
curl -O https://bitcoin.org/bin/bitcoin-core-22.0/bitcoin-22.0-osx64.tar.gz
tar zxvf bitcoin-22.0-osx64.tar.gz
cd bitcoin-22.0-osx64
```
将`bitcoin-22.0-osx64/bin`目录配置到path中，便于全局执行。
修改其中`bitcoin.conf`配置文件:
```
[regtest]
dbcache=10240
txindex=1
upnp=1
rpcuser=username
rpcauth=username:hash
rpcpassword=password
daemon=1
server=1
rest=1
rpcallowip=0.0.0.0/0
#local testnet
fallbackfee=0.0001
datadir=/Users/cjf/bitcoin/data
```
其中`rpcuser`对应的`username`可以自己指定,`rpcpassword`和`rpcauth`则需要运行(这一步比较坑，bitcoin官方文档也没说清楚...):
```
python3 ./share/rpcauth/rpcauth.py username
```

启动本地测试版本,bitcoin共有4个网络可选(main,test,signet,regtest):
```
bitcoind -regtest
```
然后最新版bitcoind默认启动后还不会自动生成钱包，这点也是个坑，[官网文档](https://developer.bitcoin.org/reference/intro.html)根本没说，因此还需要自己生成钱包:

```
bitcoin-cli -regtest createwallet walletname
bitcoin-cli -regtest loadwallet walletname
```
> 注意walletname尽量不要指定为ord，否则后面会与ord默认创建的钱包名冲突，导致需要额外配置，比较麻烦。

然后mint100个区块，给自己发50个btc:
```
bitcoin-cli -regtest -rpcwallet=test -generate 100
bitcoin-cli -regtest -rpcwallet=test -generate 1  #由于本地环境上述命令不会自动mint下一个区块，需要手动mint一个块以确认上述区块
bitcoin-cli -regtest -rpcwallet=test getbalance
```

## Mint Ordinal NFT 

安装并检查:
```
curl --proto '=https' --tlsv1.2 -fsLS https://ordinals.com/install.sh | bash -s
ord -version  #0.4.2
```

创建ord的bitcoin钱包；
```
ord --cookie-file .cookie -r wallet create
```
其中`.cookie`由自己创建，里面写入`bitcoin.conf`中的`username:password`，注意不要有换行符或空格;`-r`表示是`regtest`本地测试网。如：`btc:AaAC6QHHktRkQuXBWWlkfyOGQX4SoY_M1XtkqB5uPrs=`。这一步也很坑，官方也没怎么说清楚，如果不指定`--cookie-file`,会提示如下错误:
```
error: failed to connect to Bitcoin Core RPC at 127.0.0.1:18443/wallet/ord
because: I/O error: No such file or directory (os error 2)
```

由于mint Ordinal NFT也需要像正常的btc交易一样花费btc，因此需要先创建个地址:
```
ord --cookie-file .cookie -r wallet receive
# output your_addr
```
然后用`bitcoin-cli`给这个地址发送点btc:
```
bitcoin-cli -regtest -rpcwallet=test  sendtoaddress your_addr 10
```
可以看到该交易id:
```
ord --cookie-file .cookie -r wallet transactions
#output tx id
```
生成一个区块以确认该交易:
```
bitcoin-cli -regtest -rpcwallet=test -generate 1
```
然后随意指定一个图片或文字文件`FILE`，mint该NFT:
```
ord --cookie-file .cookie -r wallet inscribe FILE
```
前文提到会生成两笔交易，因此上述输出类似于:
```
{
  "commit": "7043fecea8b46d0542262d48cd166883346cd302d610ab78b74a9a749c914b63",
  "inscription": "00a47525e9378ed5f92f7e0498199e7ac183fd854dddf688ab573eb6d424b642i0",
  "reveal": "00a47525e9378ed5f92f7e0498199e7ac183fd854dddf688ab573eb6d424b642",
  "fees": 23008
}
```
同样，生成2个区块确认交易；
```
bitcoin-cli -regtest -rpcwallet=test -generate 2
```

然后可以查看该NFT信息:
```
ord --cookie-file .cookie -r wallet inscriptions
```
该输出类似于:
```
[
  {
    "inscription": "00a47525e9378ed5f92f7e0498199e7ac183fd854dddf688ab573eb6d424b642i0",
    "location": "00a47525e9378ed5f92f7e0498199e7ac183fd854dddf688ab573eb6d424b642:0:0",
    "explorer": "http://localhost/inscription/00a47525e9378ed5f92f7e0498199e7ac183fd854dddf688ab573eb6d424b642i0"
  }
]
```
运行`ord --cookie-file .cookie -r server`就可以在上述`explorer`链接中查看mint的NFT内容。

# 与以太坊NFT的对比
|  Ordinal Digital Artifacts   | Etherum-based NFT  |
|  ----  | ----  |
| 数据完全上链  | 除了Uniswap等一些NFT外，大多数都存储在中心化服务器或第三方去中心化存储上 |
| 费用较低，它像普通的交易一样安全地mint，且可借助于隔离见证实现手续费缩减  | 每一种NFT都需要创建一个合约，gas费很高，且每一个NFT合约都需要审计等成本很高 |
| 抗审查，不可篡改  | 需要考虑NFT合约的admin key权限问题，存在被修改的风险 |
| 完全无许可，通过tapscript即可实现mint和sell功能，不需要向合约授权  | 需要向mint合约授权，sell需要向opensea等授权,风险很高 |

# 主要参考资料
- [Ordinal Handbook](https://docs.ordinals.com/introduction.html)
- [Ordinal Number Specification](https://github.com/casey/ord/blob/master/bip.mediawiki#specification)
- [Taproot transactions](https://docs.lightning.engineering/the-lightning-network/taro/taro-protocol)
- [constructing-and-spending-taproot-outputs](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki#constructing-and-spending-taproot-outputs)
- [tapscript-example-with-tap](https://github.com/bitcoin-core/btcdeb/blob/master/doc/tapscript-example-with-tap.md)
- [segregated-witness](https://medium.com/softblocks/segregated-witness-for-dummies-d00606e8de63)