// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This is the client program that covers the following tasks:
/// 1. File I/O. Read the config file and state files for initialization, dump the state files, etc.
/// 2. Read user input (using terminal UI) about transaction creation or quitting.
/// 3. Display the status and logs to the user (using terminal UI).
/// 4. IPC communication with the bin_nakamoto and the bin_wallet processes.
use seccompiler;
use seccompiler::{BpfMap, BpfProgram};
use std::convert::TryInto;

use tui::{backend::CrosstermBackend, Terminal};
use tui_textarea::{Input, Key};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{Arc, Mutex};
use std::{
    thread,
    time::{Duration, Instant},
};

use std::io::BufWriter;
use std::{fs, string};

mod app;

/// The enum type for the IPC messages (requests) from this client to the bin_nakamoto process.
/// It is the same as the `IPCMessageRequest` enum type in the bin_nakamoto process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReqNakamoto {
    Initialize(String, String, String),
    GetAddressBalance(String),
    PublishTx(String, String),
    RequestBlock(String),
    RequestNetStatus,
    RequestChainStatus,
    RequestMinerStatus,
    RequestTxPoolStatus,
    RequestStateSerialization,
    Quit,
}

/// The enum type for the IPC messages (responses) from the bin_nakamoto process to this client.
/// It is the same as the enum type in the bin_nakamoto process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageRespNakamoto {
    Initialized,
    PublishTxDone,
    AddressBalance(String, i64),
    BlockData(String),
    NetStatus(BTreeMap<String, String>),
    ChainStatus(BTreeMap<String, String>),
    MinerStatus(BTreeMap<String, String>),
    TxPoolStatus(BTreeMap<String, String>),
    StateSerialization(String, String),
    Quitting,
    Notify(String),
}

/// The enum type for the IPC messages (requests) from this client to the bin_wallet process.
/// It is the same as the enum type in the bin_wallet process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReqWallet {
    Initialize(String),
    Quit,
    SignRequest(String),
    VerifyRequest(String, String),
    GetUserInfo,
}

/// The enum type for the IPC messages (responses) from the bin_wallet process to this client.
/// It is the same as the enum type in the bin_wallet process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageRespWallet {
    Initialized,
    Quitting,
    SignResponse(String, String),
    VerifyResponse(bool, String),
    UserInfo(String, String),
}

/// The enum type representing bot commands for controlling the client automatically.
/// The commands are read from a file or a named pipe and then executed by the client.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum BotCommand {
    /// Send a transaction message from the default user_id of the client to the given receiver_user_id, e.g, Send(`receiver_user_id`, `transaction_message`)
    Send(String, String),
    /// Wait for the given number of milliseconds, e.g., SleepMs(`milliseconds`)
    SleepMs(u64),
}

/// Read a file and return the content as a string.
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath).expect(&("Cannot read ".to_owned() + filepath));
    contents
}

/// A flag indicating whether to disable the UI thread if you need to check some debugging outputs that is covered by the UI.
/// Eventually this should be set to false and you shouldn't output debugging information directly to stdout or stderr.
const NO_UI_DEBUG_NODE: bool = false;

fn main() {
    // The usage of bin_client is as follows:
    // bin_client <client_seccomp_path> <nakamoto_config_path> <nakamoto_seccomp_path> <wallet_config_path> <wallet_seccomp_path> [<bot_command_path>]
    // - `client_seccomp_path`: The path to the seccomp file for this client process for Part B. (You can set this argument to any value during Part A.)
    // - `nakamoto_config_path`: The path to the config folder for the bin_nakamoto process. For example, `./tests/nakamoto_config1`. Your program should read the 3 files in the config folder (`BlockTree.json`, `Config.json`, `TxPool.json`) for initializing bin_nakamoto.
    // - `nakamoto_seccomp_path`: The path to the seccomp file for the bin_nakamoto process for Part B. (You can set this argument to any value during Part A.)
    // - `wallet_config_path`: The path to the config file for the bin_wallet process. For example, `./tests/_secrets/Wallet.A.json`. Your program should read the file for initializing bin_wallet.
    // - `wallet_seccomp_path`: The path to the seccomp file for the bin_wallet process for Part B. (You can set this argument to any value during Part A.)
    // - [`bot_command_path`]: *Optional* argument. The path to the file or named pipe for the bot commands. If this argument is provided, your program should read commands line-by-line from the file.
    //                         an example file of the bot commands can be found at `./tests/_bots/botA-0.jsonl`. You can also look at `run_four.sh` for an example of using the named pipe version of this argument.
    //                         The bot commands are executed by the client in the order they are read from the file or the named pipe.
    //                         The bot commands should be executed in a separate thread so that the UI thread can still be responsive.

    // Please fill in the blank
    // add in variables to keep the arguments provided when the program starts?
    let command_line_args: Arc<Vec<String>> = Arc::new(std::env::args().collect());

    if command_line_args.len() < 6 {
        panic!("not enough arguments")
    }
    let client_seccomp_path = &command_line_args[1];
    let nakamoto_config_path = &command_line_args[2];
    let nakamoto_seccomp_path = &command_line_args[3];
    let wallet_config_path = &command_line_args[4];
    let wallet_seccomp_path = &command_line_args[5];
    let mut bot_command_path = "";

    if command_line_args.len() == 7 {
        bot_command_path = &command_line_args[6];
    }

    let mut nakamoto_process = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("bin_nakamoto")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start nakamoto_process");

    thread::sleep(Duration::from_millis(500));

    let mut bin_wallet_process = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("bin_wallet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start wallet_process");

    thread::sleep(Duration::from_millis(500));

    // - Get stdin and stdout of those processes
    // - Create buffer readers if necessary
    let nakamoto_stdin_mutex = Arc::new(Mutex::new(nakamoto_process.stdin.take().unwrap()));
    let nakamoto_stdout_mutex = Arc::new(Mutex::new(BufReader::new(
        nakamoto_process.stdout.take().unwrap(),
    )));
    let nakamoto_stderr_mutex = Arc::new(Mutex::new(BufReader::new(
        nakamoto_process.stderr.take().unwrap(),
    )));
    let wallet_stdin_mutex = Arc::new(Mutex::new(bin_wallet_process.stdin.take().unwrap()));
    let wallet_stdout_mutex = Arc::new(Mutex::new(BufReader::new(
        bin_wallet_process.stdout.take().unwrap(),
    )));
    let wallet_stderr_mutex = Arc::new(Mutex::new(BufReader::new(
        bin_wallet_process.stderr.take().unwrap(),
    )));

    // - Send initialization requests to bin_nakamoto and bin_wallet:
    // send initialize request to bin_nakamoto
    let nakamoto_initialization_message = IPCMessageReqNakamoto::Initialize(
        read_string_from_file(&(nakamoto_config_path.to_owned().to_string() + "/BlockTree.json"))
            .replace("\r\n", "\n"),
        read_string_from_file(&(nakamoto_config_path.to_owned().to_string() + "/TxPool.json"))
            .replace("\r\n", "\n"),
        read_string_from_file(&(nakamoto_config_path.to_owned().to_string() + "/Config.json"))
            .replace("\r\n", "\n"),
    );
    let nakamoto_json = serde_json::to_string(&nakamoto_initialization_message).unwrap();
    {
        nakamoto_stdin_mutex
            .lock()
            .unwrap()
            .write_all((nakamoto_json + "\n").as_bytes())
            .unwrap();
    }

    // receive initialization response from nakamoto
    let mut nakamoto_buffer = String::new();
    loop {
        {
            nakamoto_stdout_mutex
                .lock()
                .unwrap()
                .read_line(&mut nakamoto_buffer)
                .unwrap();
        }
        if nakamoto_buffer.contains("Initialized") {
            break;
        }

        nakamoto_buffer.clear();
    }

    // send initialize request to bin_wallet
    let wallet_json: String;
    let initialize_wallet_string = read_string_from_file(wallet_config_path).replace("\r\n", "\n");
    let mut to_send_wallet =
        serde_json::to_string(&IPCMessageReqWallet::Initialize(initialize_wallet_string)).unwrap();
    to_send_wallet.push_str("\n");
    {
        wallet_stdin_mutex
            .lock()
            .unwrap()
            .write_all(to_send_wallet.as_bytes())
            .unwrap();
    }

    // receive initialization response from wallet
    let mut wallet_buffer = String::new();
    loop {
        {
            wallet_stdout_mutex
                .lock()
                .unwrap()
                .read_line(&mut wallet_buffer)
                .unwrap();
        }
        if wallet_buffer.contains("Initialized") {
            break;
        }

        wallet_buffer.clear();
    }

    // SECCOMP STUFF
    // seccomp stuff below

    // let client_seccomp_path = std::env::args()
    //     .nth(1)
    //     .expect("Please specify client seccomp path");
    // // Please fill in the blank
    // // sandboxing the bin_client (For part B). Leave it blank for part A.
    // let policy_str = read_string_from_file(&client_seccomp_path);
    // let filter_map: BpfMap = seccompiler::compile_from_json(
    //     policy_str.as_bytes(),
    //     std::env::consts::ARCH.try_into().unwrap(),
    // )
    // .unwrap();
    // let filter = filter_map.get("main_thread").unwrap();

    // seccompiler::apply_filter(&filter).unwrap();

    let mut user_name: String = "".to_string();
    let mut user_id: String = "".to_string();
    // Please fill in the blank
    // Read the user info from wallet

    // send get user info request to bin_wallet
    let wallet_get_info_message = IPCMessageReqWallet::GetUserInfo;
    wallet_json = serde_json::to_string(&wallet_get_info_message).unwrap();
    {
        wallet_stdin_mutex
            .lock()
            .unwrap()
            .write_all((wallet_json + "\n").as_bytes())
            .unwrap();
    }

    // recieve wallet IPC reponse for user info
    wallet_buffer = String::new();
    loop {
        {
            wallet_stdout_mutex
                .lock()
                .unwrap()
                .read_line(&mut wallet_buffer)
                .unwrap();
        }
        if wallet_buffer.contains("UserInfo") {
            break;
        }

        wallet_buffer.clear();
    }
    let wallet_user_info = serde_json::from_str(&wallet_buffer).unwrap();
    if let IPCMessageRespWallet::UserInfo(username, userid) = wallet_user_info {
        user_name = username;
        user_id = userid;
    }

    // Create the Terminal UI app
    let app_arc = Arc::new(Mutex::new(app::App::new(
        user_name.clone(),
        user_id.clone(),
        "".to_string(),
        format!("SEND $100   // By {}", user_name),
    )));

    // An enclosure func to generate signing requests when creating new transactions.
    let create_sign_req = |sender: String, receiver: String, message: String| {
        let timestamped_message = format!(
            "{}   // {}",
            message,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let sign_req = IPCMessageReqWallet::SignRequest(
            serde_json::to_string(&(sender, receiver, timestamped_message)).unwrap(),
        );
        let mut sign_req_str = serde_json::to_string(&sign_req).unwrap();
        sign_req_str.push('\n');
        return sign_req_str;
    };

    if std::env::args().len() != 6 {
        // Then there must be 7 arguments provided. The last argument is the bot commands path
        // Please fill in the blank
        // Create a thread to read the bot commands from `bot_command_path`, execute those commands and update the UI
        // Notice that the `SleepMs(1000)` doesn't mean that the all threads in the whole process should sleep for 1000ms. It means that
        // The next bot command that fakes the user interaction should be processed 1000ms later.
        // It should not block the execution of any other threads or the main thread.

        let user_id_clone = user_id.clone();
        let wallet_stdin_clone = wallet_stdin_mutex.clone();

        // link and open the bot file as stated in `bot_command_path`
        let bot_actions = read_string_from_file(bot_command_path).replace("\r\n", "\n");

        // create thread
        let _bot_handle = thread::spawn(move || {
            for line in bot_actions.split("\n") {
                if !line.contains("SleepMs") && !line.contains("Send") {
                    continue;
                }

                let command_action: BotCommand = serde_json::from_str(&line).unwrap();

                match command_action {
                    BotCommand::SleepMs(value) => {
                        thread::sleep(Duration::from_millis(value));
                    }
                    BotCommand::Send(receiver_id, transaction_message) => {
                        let user_id_clone_extra = user_id_clone.clone();
                        let sign_req_str =
                            create_sign_req(user_id_clone_extra, receiver_id, transaction_message);
                        // send sign request to wallet
                        {
                            wallet_stdin_clone
                                .lock()
                                .unwrap()
                                .write_all(sign_req_str.as_bytes())
                                .unwrap();
                        }
                    }
                }
            }
        });
    }

    // Please fill in the blank
    // - Spawn threads to read/write from/to bin_nakamoto/bin_wallet. (Through their piped stdin and stdout)
    // - You should request for status update from bin_nakamoto periodically (every 500ms at least) to update the App (UI struct) accordingly.
    // - You can also create threads to read from stderr of bin_nakamoto/bin_wallet and add those lines to the UI (app.stderr_log) for easier debugging.

    let wallet_stdout_clone = wallet_stdout_mutex.clone();
    let nakamoto_stdin_clone = nakamoto_stdin_mutex.clone();

    // read wallet stdout to receive sign response, send to nakamoto to publish tx
    let handle_wallet_nakamoto_publish_tx = thread::spawn(move || {
        loop {
            let mut wallet_sign_buffer = String::new();
            loop {
                {
                    wallet_stdout_clone
                        .lock()
                        .unwrap()
                        .read_line(&mut wallet_sign_buffer)
                        .unwrap();
                }
                if wallet_sign_buffer.contains("SignResponse") {
                    break;
                }

                wallet_sign_buffer.clear();
            }

            // send publishTx to nakamoto
            let wallet_sign_response: IPCMessageRespWallet =
                serde_json::from_str(&wallet_sign_buffer).unwrap();
            if let IPCMessageRespWallet::SignResponse(data, signature) = wallet_sign_response {
                let nakamoto_ipc_req = IPCMessageReqNakamoto::PublishTx(data, signature); // make publishTx
                let mut nakamoto_json = serde_json::to_string(&nakamoto_ipc_req).unwrap();
                nakamoto_json.push_str("\n");
                //let mut nakamoto_stdin_clone_lock = nakamoto_stdin_clone.lock().unwrap();
                nakamoto_stdin_clone
                    .lock()
                    .unwrap()
                    .write_all(nakamoto_json.as_bytes())
                    .unwrap();
            }
        }
    });
    // handle_processes_communicate.join().unwrap(); // shifted below

    let user_id_clone2 = user_id.clone();
    let nakamoto_stdin_clone = nakamoto_stdin_mutex.clone();
    let nakamoto_config_path_clone = nakamoto_config_path.clone();

    let handle_send_status_updates = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));

            // send ipc request message to nakamoto to get address balance
            let mut nakamoto_ipc_req =
                IPCMessageReqNakamoto::GetAddressBalance((user_id_clone2).to_string());
            let mut nakamoto_json = serde_json::to_string(&nakamoto_ipc_req).unwrap();
            nakamoto_json.push_str("\n");
            {
                nakamoto_stdin_clone
                    .lock()
                    .unwrap()
                    .write_all(nakamoto_json.as_bytes())
                    .unwrap();
            }
            // send ipc request message to nakamoto to get netStatus
            nakamoto_ipc_req = IPCMessageReqNakamoto::RequestNetStatus;
            nakamoto_json = serde_json::to_string(&nakamoto_ipc_req).unwrap();
            nakamoto_json.push_str("\n");
            {
                nakamoto_stdin_clone
                    .lock()
                    .unwrap()
                    .write_all(nakamoto_json.as_bytes())
                    .unwrap();
            }

            // send ipc request message to nakamoto to get chainStatus
            nakamoto_ipc_req = IPCMessageReqNakamoto::RequestChainStatus;
            nakamoto_json = serde_json::to_string(&nakamoto_ipc_req).unwrap();
            nakamoto_json.push_str("\n");
            {
                nakamoto_stdin_clone
                    .lock()
                    .unwrap()
                    .write_all(nakamoto_json.as_bytes())
                    .unwrap();
            }

            // send ipc request message to nakamoto to get minerStatus
            nakamoto_ipc_req = IPCMessageReqNakamoto::RequestMinerStatus;
            nakamoto_json = serde_json::to_string(&nakamoto_ipc_req).unwrap();
            nakamoto_json.push_str("\n");
            {
                nakamoto_stdin_clone
                    .lock()
                    .unwrap()
                    .write_all(nakamoto_json.as_bytes())
                    .unwrap();
            }

            // send ipc request message to nakamoto to get txPoolStatus
            nakamoto_ipc_req = IPCMessageReqNakamoto::RequestTxPoolStatus;
            nakamoto_json = serde_json::to_string(&nakamoto_ipc_req).unwrap();
            nakamoto_json.push_str("\n");
            {
                nakamoto_stdin_clone
                    .lock()
                    .unwrap()
                    .write_all(nakamoto_json.as_bytes())
                    .unwrap();
            }
        }
    });

    let nakamoto_stdout_clone = nakamoto_stdout_mutex.clone();
    let app_ui_ref_status = app_arc.clone();

    let handle_read_nakamoto_stdout = thread::spawn(move || {
        loop {
            let mut nakamoto_reply = String::new();
            loop {
                {
                    nakamoto_stdout_clone
                        .lock()
                        .unwrap()
                        .read_line(&mut nakamoto_reply)
                        .unwrap();
                }
                if nakamoto_reply.contains("Initialized")
                    || nakamoto_reply.contains("PublishTxDone")
                    || nakamoto_reply.contains("AddressBalance")
                    || nakamoto_reply.contains("BlockData")
                    || nakamoto_reply.contains("NetStatus")
                    || nakamoto_reply.contains("ChainStatus")
                    || nakamoto_reply.contains("MinerStatus")
                    || nakamoto_reply.contains("TxPoolStatus")
                    || nakamoto_reply.contains("StateSerialization")
                    || nakamoto_reply.contains("Quitting")
                    || nakamoto_reply.contains("Notify")
                {
                    // println!("{}", nakamoto_reply);
                    break;
                }
                nakamoto_reply.clear();
            }

            let nakamoto_enum: IPCMessageRespNakamoto =
                serde_json::from_str(&nakamoto_reply).unwrap();
            match nakamoto_enum {
                IPCMessageRespNakamoto::AddressBalance(user_id, balance) => {
                    if user_id == app_ui_ref_status.lock().unwrap().user_id {
                        app_ui_ref_status.lock().unwrap().user_balance = balance;
                    }
                }
                IPCMessageRespNakamoto::NetStatus(map) => {
                    {
                        app_ui_ref_status.lock().unwrap().network_status.insert(
                            "#address".to_string(),
                            serde_json::to_string(&map.get("#address"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().network_status.insert(
                            "#recv_msg".to_string(),
                            serde_json::to_string(&map.get("#recv_msg"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().network_status.insert(
                            "#send_msg".to_string(),
                            serde_json::to_string(&map.get("#send_msg"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                }
                IPCMessageRespNakamoto::ChainStatus(map) => {
                    {
                        app_ui_ref_status.lock().unwrap().blocktree_status.insert(
                            "#blocks".to_string(),
                            serde_json::to_string(&map.get("#blocks"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().blocktree_status.insert(
                            "#orphans".to_string(),
                            serde_json::to_string(&map.get("#orphans"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().blocktree_status.insert(
                            "finalized_id".to_string(),
                            serde_json::to_string(&map.get("finalized_id"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().blocktree_status.insert(
                            "root_id".to_string(),
                            serde_json::to_string(&map.get("root_id"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().blocktree_status.insert(
                            "working_depth".to_string(),
                            serde_json::to_string(&map.get("working_depth"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().blocktree_status.insert(
                            "working_id".to_string(),
                            serde_json::to_string(&map.get("working_id"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                }
                IPCMessageRespNakamoto::MinerStatus(map) => {
                    {
                        app_ui_ref_status.lock().unwrap().miner_status.insert(
                            "#thread".to_string(),
                            serde_json::to_string(&map.get("#thread"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().miner_status.insert(
                            "difficulty".to_string(),
                            serde_json::to_string(&map.get("difficulty"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().miner_status.insert(
                            "is_running".to_string(),
                            serde_json::to_string(&map.get("is_running"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                }
                IPCMessageRespNakamoto::TxPoolStatus(map) => {
                    {
                        app_ui_ref_status.lock().unwrap().txpool_status.insert(
                            "#pool_tx_map".to_string(),
                            serde_json::to_string(&map.get("#pool_tx_map"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().txpool_status.insert(
                            "#removed_tx_ids".to_string(),
                            serde_json::to_string(&map.get("#removed_tx_ids"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                    {
                        app_ui_ref_status.lock().unwrap().txpool_status.insert(
                            "#pool_tx_ids".to_string(),
                            serde_json::to_string(&map.get("#pool_tx_ids"))
                                .unwrap()
                                .replace(r#"""#, ""),
                        );
                    }
                }
                IPCMessageRespNakamoto::StateSerialization(
                    blocktree_json_string,
                    tx_pool_json_string,
                ) => {
                    // save into json file, with the config path, appended with timestamps
                    let time_stamp_json = serde_json::to_string(&SystemTime::now()).unwrap();
                    #[derive(Serialize, Deserialize)]
                    struct Time {
                        secs_since_epoch: i64,
                        nanos_since_epoch: i64,
                    }
                    let time_stamp: Time = serde_json::from_str(&time_stamp_json).unwrap();
                    let time_append: String = time_stamp.secs_since_epoch.to_string();

                    let blocktree_json_filename = nakamoto_config_path_clone.to_owned().to_string()
                        + r#"/"#
                        + &time_append
                        + "-BlockTree.json";
                    let tx_pool_json_filename = nakamoto_config_path_clone.to_owned().to_string()
                        + r#"/"#
                        + &time_append
                        + "-TxPool.json";

                    let blocktree_json_filename_clone = blocktree_json_filename.clone();
                    let tx_pool_json_filename_clone = tx_pool_json_filename.clone();
                    let mut _blocktree_file = File::create(blocktree_json_filename);
                    fs::write(blocktree_json_filename_clone, blocktree_json_string)
                        .expect("Failed to write to blocktree file");
                    let mut _tx_pool_file = File::create(tx_pool_json_filename);
                    fs::write(tx_pool_json_filename_clone, tx_pool_json_string)
                        .expect("Failed to write to tx pool file");
                }
                IPCMessageRespNakamoto::PublishTxDone => {
                    // do nothing
                }
                IPCMessageRespNakamoto::Initialized => {
                    // do nothing
                }
                IPCMessageRespNakamoto::Quitting => {
                    // do nothing
                }
                IPCMessageRespNakamoto::Notify(message) => {
                    app_ui_ref_status.lock().unwrap().notify_log.push(message);
                }
                IPCMessageRespNakamoto::BlockData(_block_data) => {
                    // do nothing
                }
            }
        }
    });
    // handle_read_nakamoto_stdout.join().unwrap(); // shifted below

    let nakamoto_stderr_clone = nakamoto_stderr_mutex.clone();
    let app_ui_ref_err = app_arc.clone();

    let handle_nakamoto_stderr = thread::spawn(move || {
        let mut nakamoto_err_ignore = String::new();
        loop {
            {
                nakamoto_stderr_clone
                    .lock()
                    .unwrap()
                    .read_line(&mut nakamoto_err_ignore)
                    .unwrap();
            }
            if nakamoto_err_ignore
                == "To only capture error messages from nakamoto from this instance onwards"
            {
                break;
            }
            nakamoto_err_ignore.clear();
        }
        loop {
            let mut nakamoto_err_message = String::new();
            loop {
                {
                    nakamoto_stderr_clone
                        .lock()
                        .unwrap()
                        .read_line(&mut nakamoto_err_message)
                        .unwrap();
                }
                if !nakamoto_err_message.is_empty() {
                    break;
                }
                nakamoto_err_message.clear();
            }
            app_ui_ref_err
                .lock()
                .unwrap()
                .stderr_log
                .push(nakamoto_err_message);
        }
    }); // handle_nakamoto_stderr.join().unwrap(); // shifted below

    // UI thread. Modify it to suit your needs.
    let app_ui_ref = app_arc.clone();
    // let bin_wallet_stdin_p_cloned = bin_wallet_stdin_p.clone();
    // let nakamoto_stdin_p_cloned = nakamoto_stdin_p.clone();
    let bin_wallet_stdin_p_cloned = wallet_stdin_mutex.clone();
    let nakamoto_stdin_p_cloned = nakamoto_stdin_mutex.clone();
    let handle_ui = thread::spawn(move || {
        let tick_rate = Duration::from_millis(200);
        if NO_UI_DEBUG_NODE {
            // If app_ui.should_quit is set to true, the UI thread will exit.
            loop {
                if app_ui_ref.lock().unwrap().should_quit {
                    break;
                }
                // sleep for 500ms
                thread::sleep(Duration::from_millis(500));
            }
            return;
        }
        let ui_loop = || -> Result<(), io::Error> {
            // setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let mut last_tick = Instant::now();
            loop {
                terminal.draw(|f| app_ui_ref.lock().unwrap().draw(f))?;

                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_millis(100));

                if crossterm::event::poll(timeout)? {
                    let input = event::read()?.into();
                    let mut app = app_ui_ref.lock().unwrap();
                    match input {
                        Input { key: Key::Esc, .. } => {
                            app.on_quit();
                        }
                        Input { key: Key::Down, .. } => app.on_down(),
                        Input { key: Key::Up, .. } => app.on_up(),
                        Input {
                            key: Key::Enter, ..
                        } => {
                            if !app.are_inputs_valid {
                                app.client_log("Invalid inputs! Cannot create Tx.".to_string());
                            } else {
                                let (sender, receiver, message) = app.on_enter();
                                let sign_req_str = create_sign_req(sender, receiver, message);
                                bin_wallet_stdin_p_cloned
                                    .lock()
                                    .unwrap()
                                    .write_all(sign_req_str.as_bytes())
                                    .unwrap();
                            }
                        }
                        // on control + s, request Nakamoto to serialize its state
                        Input {
                            key: Key::Char('s'),
                            ctrl: true,
                            ..
                        } => {
                            let serialize_req = IPCMessageReqNakamoto::RequestStateSerialization;
                            let mut to_send = serde_json::to_string(&serialize_req).unwrap();
                            to_send.push_str("\n");
                            nakamoto_stdin_p_cloned
                                .lock()
                                .unwrap()
                                .write_all(to_send.as_bytes())
                                .unwrap();
                        }
                        input => {
                            app.on_textarea_input(input);
                        }
                    }
                }

                let mut app = app_ui_ref.lock().unwrap();
                if last_tick.elapsed() >= tick_rate {
                    app.on_tick();
                    last_tick = Instant::now();
                }
                if app.should_quit {
                    break;
                }
            }
            // restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            Ok(())
        };
        ui_loop().unwrap();
    });
    handle_ui.join().unwrap();

    eprintln!("--- Sending \"Quit\" command...");
    // nakamoto_stdin_p.lock().unwrap().write_all("\"Quit\"\n".as_bytes()).unwrap();
    // bin_wallet_stdin_p.lock().unwrap().write_all("\"Quit\"\n".as_bytes()).unwrap();
    nakamoto_stdin_mutex
        .lock()
        .unwrap()
        .write_all("\"Quit\"\n".as_bytes())
        .unwrap();
    wallet_stdin_mutex
        .lock()
        .unwrap()
        .write_all("\"Quit\"\n".as_bytes())
        .unwrap();

    // Please fill in the blank
    // Wait for the IPC threads to finish
    handle_wallet_nakamoto_publish_tx.join().unwrap();
    handle_send_status_updates.join().unwrap();
    handle_read_nakamoto_stdout.join().unwrap();
    handle_nakamoto_stderr.join().unwrap();

    let ecode1 = nakamoto_process
        .wait()
        .expect("failed to wait on child nakamoto");
    eprintln!("--- nakamoto ecode: {}", ecode1);

    let ecode2 = bin_wallet_process
        .wait()
        .expect("failed to wait on child bin_wallet");
    eprintln!("--- bin_wallet ecode: {}", ecode2);
}
