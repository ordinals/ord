#!/bin/bash


counter=1
while [ $counter -le 200 ]
do
  cargo run -- --chain regtest --wallet ord1 wallet inscribe --fee-rate 1 ./test.json;
  if [ $((counter % 5)) == 0 ]
  then
    bitcoin-cli -rpcwallet="cmion" -generate 201
    bitcoin-cli -rpcwallet="cmion" sendtoaddress bcrt1p3wqzrhe3rp4fg3k0vmug5u29cknf2q4ljv4tpcdlptc0apk80kzqkdhupc 1000
    pwd
  fi
  ((counter++))
done



#Balance: cargo run -- --chain regtest --wallet ord1 wallet balance
#Deposit Cardinal: bitcoin-cli -rpcwallet="cmion" sendtoaddress bcrt1p3wqzrhe3rp4fg3k0vmug5u29cknf2q4ljv4tpcdlptc0apk80kzqkdhupc 100
