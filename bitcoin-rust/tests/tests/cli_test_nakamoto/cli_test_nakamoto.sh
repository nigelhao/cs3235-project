cd ../../
cat ./tests/cli_test_nakamoto/cli_test_nakamoto_0.input | ./target/debug/bin_nakamoto ./bin_client/policies/seccomp_nakamoto.json > ./tests/cli_test_nakamoto/cli_test_nakamoto_0.output
diff ./tests/cli_test_nakamoto/cli_test_nakamoto_0.output ./tests/cli_test_nakamoto/cli_test_nakamoto_0.output.expected
