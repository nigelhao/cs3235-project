# `bitcoin-rust` CS3235 Project Template

Please look at the released PDF document on Canvas for the full description of the project.

As mentioned in the project description, further clarification (if any) will be in this [Clarification Document](https://docs.google.com/presentation/d/1nK-LnnJtKO0bSBarg0smIWebgA6R82q6NOlYwx_o-0s/edit?usp=sharing).

This README describes the files and directories in this template. It also includes some related links and commands that you may find useful.

## Files and Directories

### Packages 
This project template contains 7 packages, including 3 executable packages and 4 library packages. The executable packages are `bin_client`, `bin_nakamoto`, and `bin_wallet`. The library packages are `lib_chain`, `lib_miner`, `lib_network`, and `lib_tx_pool`. The `bin_nakamoto` package depends on the 4 library packages. 

### Seccomp Policies
For part B, you can find the seccomp policies in the `bin_client/policies` directory. The folder contains the following files:

- `seccomp_all_allow.json`: a policy that allows all syscalls
- `seccomp_all_deny.json`: a policy that denies all syscalls
- `seccomp_client.json`: a policy that allows syscalls used by the `bin_client` program (after applying the seccomp filter)
- `seccomp_nakamoto.json`: a policy that allows syscalls used by the `bin_nakamoto` program (after applying the seccomp filter)
- `seccomp_wallet.json`: a policy that allows syscalls used by the `bin_wallet` program (after applying the seccomp filter)

Please do not modify the filenames of the seccomp policies. The filenames are used by the `bin_client` program in the `start_four.sh` script to load the correct policy. Those files will also be read by `random_policy_gen.py` during your video demonstration. The files `seccomp_all_allow.json` and `seccomp_all_deny.json` are provided for debugging purposes.

### Tests

Under the tests folder, you can find the following sub-folders:

- `_bots`: contains the files for bot commands. Bot commands are used to automate the `bin_client` program for publishing transactions (so that there are always new transactions in the bitcoin network during your video demonstration). There are two types of files providing bot commands:
  - `bot*.jsonl`: A file where each line is a JSON object containing a bot command. If provided as the last argument to the `bin_client` program, it will read those commands and execute them.
  - `bot*.py`: A python script that write infinite number of bot commands to stdout. `botA-1.py` is used for your video demonstration. It is used in combination with *named pipe*. The script `run_four.sh` contains an example of using named pipe as a drop-in replacement of file to provide infinite number of bot commands generated in real-time.
- `_secrets`: Contains the serialized `Wallet` objects that contains the secret private keys. These files are used for initialization during your video demonstration. 
- `cli_test_nakamoto`: Some commands and files that you can use to test your `bin_nakamoto` program from the command line.
- `cli_test_wallet`: Some commands and files that you can use to test your `bin_wallet` program from the command line.
- `nakamoto_cinfig*`: Files for configuring and initializing the `bin_client` for the video demonstration.

### Other files

- `./run_four.sh`: A script that starts 4 `bin_client` programs inside a tmux session with specified configurations. It is used for your video demonstration.
- `./random_policy_gen.py`: A script that reads your seccomp policies and generate random mutations for your video demonstration.
- `./save_four.sh`: A script that sends `ctrl+s` to all 4 `bin_client` programs in the tmux session. It will instruct the `bin_client` programs to save the block tree and the transaction pool to files. It is used for your video demonstration.
- `./stop_four.sh`: A script that kills the tmux session. It is used for your video demonstration.
- `./submission_cleanup.sh`: A script that help you remove compiled binaries and other files that are not needed for submission. 


## Useful Resources

- For the Rust programming language, you can refer to the [Rust Book](https://doc.rust-lang.org/book/) and the [Rust by Example](https://doc.rust-lang.org/rust-by-example/) website.
- For the MPSC communication channels used in `P2PNetwork` or other places, you can refer to the [Rust documentation](https://doc.rust-lang.org/std/sync/mpsc/index.html), and also the [Rust book](https://doc.rust-lang.org/book/ch16-02-message-passing.html) and [Rust by Example](https://doc.rust-lang.org/rust-by-example/std_misc/channels.html) website.
- For Merkle Tree used in the `lib_chain`, you can refer to the Wikipedia page on [Merkle Tree](https://en.wikipedia.org/wiki/Merkle_tree).

