./build.sh
tmux has-session -t nakafour 2>/dev/null
if [ $? != 0 ]; then
    mkfifo botpipeA
    # cat ./tests/_bots/botA-0.jsonl > botpipeA &
    python3 ./tests/_bots/botA-1.py > botpipeA &
    NAKAMOTO_SECCOMP_PATH=./bin_client/policies/seccomp_nakamoto.json
    WALLET_SECCOMP_PATH=./bin_client/policies/seccomp_wallet.json
    CLIENT_SECCOMP_PATH=./bin_client/policies/seccomp_client.json
    tmux new-session -d -s nakafour
    tmux send-keys -t nakafour "./target/debug/bin_client ${CLIENT_SECCOMP_PATH} ./tests/nakamoto_config1 ${NAKAMOTO_SECCOMP_PATH} ./tests/_secrets/Wallet.A.json ${WALLET_SECCOMP_PATH} botpipeA" C-m
    tmux split-window -h -t nakafour
    tmux send-keys -t nakafour "./target/debug/bin_client ${CLIENT_SECCOMP_PATH} ./tests/nakamoto_config2 ${NAKAMOTO_SECCOMP_PATH} ./tests/_secrets/Wallet.B.json ${WALLET_SECCOMP_PATH}" C-m
    tmux split-window -v -t nakafour
    tmux send-keys -t nakafour "./target/debug/bin_client ${CLIENT_SECCOMP_PATH} ./tests/nakamoto_config3 ${NAKAMOTO_SECCOMP_PATH} ./tests/_secrets/Wallet.C.json ${WALLET_SECCOMP_PATH}" C-m
    tmux split-window -v -t nakafour
    tmux send-keys -t nakafour "./target/debug/bin_client ${CLIENT_SECCOMP_PATH} ./tests/nakamoto_config4 ${NAKAMOTO_SECCOMP_PATH} ./tests/_secrets/Wallet.D.json ${WALLET_SECCOMP_PATH}" C-m
    tmux select-layout -t nakafour tiled
    tmux attach-session -t nakafour
fi
