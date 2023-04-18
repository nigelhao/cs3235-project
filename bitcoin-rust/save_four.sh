# send ctrl + s to all panels in the tmux session named `nakafour`

tmux has-session -t nakafour 2>/dev/null
if [ $? != 0 ]; then
    echo "tmux session named 'nakafour' does not exist"
    exit 1
fi

tmux send-keys -t nakafour:0.0 C-s
tmux send-keys -t nakafour:0.1 C-s
tmux send-keys -t nakafour:0.2 C-s
tmux send-keys -t nakafour:0.3 C-s

