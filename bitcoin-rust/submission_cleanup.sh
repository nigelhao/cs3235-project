# remove log files
/bin/rm -rf ./tests/nakamoto_config*/*-BlockTree.json
/bin/rm -rf ./tests/nakamoto_config*/*-TxPool.json

# remove compiled binaries
/bin/rm -rf ./target
/bin/rm -rf ./bin_client/target
/bin/rm -rf ./bin_nakamoto/target
/bin/rm -rf ./bin_wallet/target
/bin/rm -rf ./lib_chain/target
/bin/rm -rf ./lib_miner/target
/bin/rm -rf ./lib_network/target
/bin/rm -rf ./lib_tx_pool/target
