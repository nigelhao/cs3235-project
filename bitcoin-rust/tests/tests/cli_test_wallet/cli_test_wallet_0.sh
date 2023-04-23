cd ../../
cat ./tests/cli_test_wallet/cli_test_wallet_0.input | ./target/debug/bin_wallet ./bin_client/policies/seccomp_wallet.json > ./tests/cli_test_wallet/cli_test_wallet_0.output
diff ./tests/cli_test_wallet/cli_test_wallet_0.output ./tests/cli_test_wallet/cli_test_wallet_0.output.expected
