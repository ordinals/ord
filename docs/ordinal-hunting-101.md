Ordinal Hunting 101
===================

Tools for the hunt:
- `bitcoind` synced with txindex=1
- `bitcoin-cli` 
- `ord` compiled and indexed 
- 

Steps:
```bash
bitcoin-cli createwallet ord-watch-only true true
bitcoin-cli loadwallet ord-watch-only 
bitcoin-cli importdescriptors '[{ "desc": "", "timestamp": } {"desc": , "timestamp": }]'

ord index 
ord wallet identify 
```

