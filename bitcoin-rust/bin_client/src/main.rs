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

use std::fs;

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
    // TODO may need to put p2p messages into UI?
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
    {
        wallet_stdout_mutex
            .lock()
            .unwrap()
            .read_line(&mut wallet_buffer)
            .unwrap();
    }
    let _wallet_status: IPCMessageRespWallet = serde_json::from_str(&wallet_buffer).unwrap();

    // The path to the seccomp file for this client process for Part B. (You can set this argument to any value during Part A.)
    let client_seccomp_path = std::env::args()
        .nth(1)
        .expect("Please specify client seccomp path");
    // Please fill in the blank
    // sandboxing the bin_client (For part B). Leave it blank for part A.
    // TODO after completing part A
    // TODO after completing part A
    // TODO after completing part A

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
    {
        wallet_stdout_mutex
            .lock()
            .unwrap()
            .read_line(&mut wallet_buffer)
            .unwrap();
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
                if line.is_empty() {
                    continue;
                }
                let command_action: BotCommand = serde_json::from_str(&line).unwrap();
                match command_action {
                    BotCommand::SleepMs(value) => {
                        thread::sleep(Duration::from_millis(value));
                    }
                    BotCommand::Send(receiver_id, transaction_message) => {
                        let user_id_clone_extra = user_id_clone.clone();
                        let mut sign_req_str =
                            create_sign_req(user_id_clone_extra, receiver_id, transaction_message);
                        sign_req_str.push_str("\n");
                        // send sign request to wallet
                        {
                            wallet_stdin_clone
                                .lock()
                                .unwrap()
                                .write_all(sign_req_str.as_bytes()) // broken pipe here /////////////////////////////////
                                .unwrap();
                        }
                    }
                }
            }
        });
        // bot_handle.join().unwrap(); // i think dont need to wait for bot_handle
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
            let mut wallet_buffer = String::new();
            loop {
                {
                    wallet_stdout_clone
                        .lock()
                        .unwrap()
                        .read_line(&mut wallet_buffer)
                        .unwrap();
                }
                if wallet_buffer != "" {
                    break;
                }

                wallet_buffer.clear();
            }

            // send publishTx to nakamoto
            let wallet_sign_response: IPCMessageRespWallet =
                serde_json::from_str(&wallet_buffer).unwrap();
            if let IPCMessageRespWallet::SignResponse(data, signature) = wallet_sign_response {
                let nakamoto_ipc_req = IPCMessageReqNakamoto::PublishTx(data, signature); // make publishTx
                let mut nakamoto_json = serde_json::to_string(&nakamoto_ipc_req).unwrap();
                nakamoto_json.push_str("\n");
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

    let handle_send_status_updates = thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(500));
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

            // send ipc request message to nakamoto to get requestStateSerialization
            nakamoto_ipc_req = IPCMessageReqNakamoto::RequestStateSerialization;
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

    // FROM HERE ONWARDS NEED TO CHECK
    let handle_read_nakamoto_stdout = thread::spawn(move || {
        // change into a loop, check whether buffer empty
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
                    break;
                }
                nakamoto_reply.clear();
            }
            // println!("{}", nakamoto_reply);

            let nakamoto_enum: IPCMessageRespNakamoto =
                serde_json::from_str(&nakamoto_reply).unwrap();
            match nakamoto_enum {
                IPCMessageRespNakamoto::AddressBalance(user_id, balance) => {
                    // println!("{} {} address balance", user_id, balance); // to figure out how to update UI
                }
                IPCMessageRespNakamoto::NetStatus(map) => {
                    // println!("{:?} netstatus", map); // to figure out how to update UI
                }
                IPCMessageRespNakamoto::ChainStatus(map) => {
                    // println!("{:?} chainstatus", map); // to figure out how to update UI
                }
                IPCMessageRespNakamoto::MinerStatus(map) => {
                    // println!("{:?} minerstatus", map); // to figure out how to update UI
                }
                IPCMessageRespNakamoto::TxPoolStatus(map) => {
                    // println!("{:?} txPoolstatus", map); // to figure out how to update UI
                }
                IPCMessageRespNakamoto::StateSerialization(
                    blocktree_json_string,
                    tx_pool_json_string,
                ) => {
                    // println!(
                    //     "{} {} stateSerialization",
                    //     blocktree_json_string, tx_pool_json_string
                    // ); // to figure out how to update UI
                }
                IPCMessageRespNakamoto::PublishTxDone => {
                    // println!("publishTxDone"); // to figure out how to update UI
                }
                IPCMessageRespNakamoto::Initialized => {
                    // do nothing
                }
                IPCMessageRespNakamoto::Quitting => {
                    // do nothing
                }
                IPCMessageRespNakamoto::Notify(_message) => {
                    // do nothing
                }
                IPCMessageRespNakamoto::BlockData(_block_data) => {
                    // do nothing
                }
            }
        }
    });
    // handle_read_nakamoto_stdout.join().unwrap(); // shifted below

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
                            let mut nakamoto_stdin = nakamoto_stdin_p_cloned.lock().unwrap();
                            let mut to_send = serde_json::to_string(&serialize_req).unwrap();
                            to_send.push_str("\n");
                            nakamoto_stdin.write_all(to_send.as_bytes()).unwrap();
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

    let ecode1 = nakamoto_process
        .wait()
        .expect("failed to wait on child nakamoto");
    eprintln!("--- nakamoto ecode: {}", ecode1);

    let ecode2 = bin_wallet_process
        .wait()
        .expect("failed to wait on child bin_wallet");
    eprintln!("--- bin_wallet ecode: {}", ecode2);
}
