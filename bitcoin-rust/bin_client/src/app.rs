// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This file implements the Terminal User Interface for the client
/// It displays the status of the blockchain, the network, the transaction pool, and the miner.
/// It also allows the user to create and publish transactions.
/// The user can use the arrow keys to navigate between the text areas and press enter to publish.
/// It also displays the logs and notifications from the client.
/// You don't have to modify this file. But you are free to change it if you like.

use std::collections::BTreeMap;

use tui::{
    backend::{Backend},
    widgets::{Widget, Block, Borders, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction, Alignment},
    Frame,
    style::{Style, Color},
};
use tui_textarea::{TextArea, Input};

/// The struct to represent the terminal user interface for the client.
pub struct App<'a> {
    /// the friendly name of the user. Will be used in the `Create Transaction` panel of the UI.
    pub friendly_user_name: String,
    /// whether the user inputs in the `Create Transaction` panel of the UI are valid or not.
    pub are_inputs_valid: bool,
    /// the user ID of the sender. Will be used as the default sender ID in the `Create Transaction` panel of the UI.
    pub user_id: String,
    /// the balance of the user. Will be displayed in the `Create Transaction` panel of the UI.
    pub user_balance: i64,
    /// whether the user wants to quit the program or not.
    pub should_quit: bool,
    /// the status of the blocktree as a dictionary of key-value pairs (for debugging purpose)
    pub blocktree_status: BTreeMap<String, String>,
    /// the status of the network as a dictionary of key-value pairs (for debugging purpose)
    pub network_status: BTreeMap<String, String>,
    /// the status of the transaction pool as a dictionary of key-value pairs (for debugging purpose)
    pub txpool_status: BTreeMap<String, String>,
    /// the status of the miner as a dictionary of key-value pairs (for debugging purpose)
    pub miner_status: BTreeMap<String, String>,
    /// the notification logs from the client for debugging purpose.
    pub notify_log: Vec<String>,
    /// the stderr logs from the client for debugging purpose.
    pub stderr_log: Vec<String>,
    /// the text areas in the UI for inputting sender Id, receiver ID and message to create a transaction.
    pub textareas: Vec<TextArea<'a>>,
    /// the index of the text area that is currently in focus.
    pub textarea_choosing_idx: usize
}

impl<'a> App<'a> {

    /// Create a new instance of the App struct.
    /// - `user_name`: the friendly name of the user. 
    /// - `default_sender`: the user ID of the sender (initial value in the input box). 
    /// - `default_receiver`: the user ID of the receiver (initial value in the input box).
    /// - `default_message`: the message to be sent (initial value in the input box).
    pub fn new(user_name: String, default_sender: String, default_receiver: String, default_message: String) -> App<'a> {        
        
        App {
            friendly_user_name: user_name,
            are_inputs_valid: false,
            user_id: default_sender.clone(),
            user_balance: -1,
            should_quit: false,
            blocktree_status: BTreeMap::new(),
            network_status: BTreeMap::new(),
            txpool_status: BTreeMap::new(),
            miner_status: BTreeMap::new(),
            notify_log: vec![],
            stderr_log: vec![],
            textareas: vec![
                App::textarea_with_title("Sender ID".to_string(), default_sender),
                App::textarea_with_title("Receiver ID".to_string(), default_receiver),
                App::textarea_with_title("Message".to_string(), default_message)
            ],
            textarea_choosing_idx: 1
        }
    }

    /// Log to the stderr log.
    pub fn client_log(&mut self, log: String) {
        self.stderr_log.push(format!("[Client({})] {}", self.friendly_user_name, log));
    }

    /// Private function to create a text area with a title.
    fn textarea_with_title(title: String, default_val: String) -> TextArea<'a> {
        let mut textarea = TextArea::new(vec![default_val]);
        textarea.set_style(Style::default().fg(Color::LightGreen));
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        );
        
        textarea
    }
    
    /// Return the values in the text areas on enter.
    pub fn on_enter(&mut self) -> (String, String, String) {
        let sender = self.textareas[0].lines()[0].clone();
        let receiver = self.textareas[1].lines()[0].clone();
        let message = self.textareas[2].lines()[0].clone();
        self.stderr_log.push(format!("[Client] ENTER pressed.  Sender: {}  Receiver: {}  Message: {}", &sender, &receiver, &message));
        (sender, receiver, message)
    }

    /// Change the focus to the text area above.
    pub fn on_up(&mut self) {
        self.textarea_choosing_idx = (self.textarea_choosing_idx + 3 - 1) % 3;
    }

    /// Change the focus to the text area below.
    pub fn on_down(&mut self) {
        self.textarea_choosing_idx = (self.textarea_choosing_idx + 3 + 1) % 3;
    }

    /// Set the values in the text areas.
    pub fn set_inputs(&mut self, receiver: Option<String>, message: Option<String>) {
        match receiver {
            Some(receiver) => {
                self.textareas[1].move_cursor(tui_textarea::CursorMove::Head);
                self.textareas[1].delete_line_by_end();
                self.textareas[1].insert_str(&receiver);
            }
            None => {}
        }
        match message {
            Some(message) => {
                self.textareas[2].move_cursor(tui_textarea::CursorMove::Head);
                self.textareas[2].delete_line_by_end();
                self.textareas[2].insert_str(&message);
            }
            None => {}
        }
    }


    /// Set the `should_quit` flag to true.
    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }


    /// Handle the input event for the text areas.
    pub fn on_textarea_input(&mut self, input: Input) {
        self.textareas[self.textarea_choosing_idx].input(input);
    }

    /// On tick event
    pub fn on_tick(&mut self) {
        // Update progress

    }

    /// Validating the textarea for user ID.
    fn validate_id_textarea(textarea: &mut TextArea, is_focus: bool) -> bool {
        if textarea.lines()[0].len() != 80 {
            if is_focus {
                textarea.set_style(Style::default().fg(Color::LightRed));
            } else {
                textarea.set_style(Style::default().fg(Color::Red));
            }
            false
        } else {
            if is_focus {
                textarea.set_style(Style::default().fg(Color::LightGreen));
            } else {
                textarea.set_style(Style::default().fg(Color::Green));
            }
            true
        }
    }

    /// Validating the textarea for message.
    fn validate_message_textarea(textarea: &mut TextArea, is_focus: bool, user_balance: i64) -> bool {
        let mut is_valid = true;
        
        if !textarea.lines()[0].starts_with("SEND $") {
            is_valid = false;
        } 
        else {
            // check if the amount after the $ is a valid number and smaller than the user's balance
            // extract "300" from " SEND $300   // By Alice"
            let amount_str = textarea.lines()[0].split("$").collect::<Vec<&str>>()[1].split(" ").collect::<Vec<&str>>()[0];
            if amount_str.parse::<i64>().is_err() {
                is_valid = false;
            } else {
                let amount = amount_str.parse::<i64>().unwrap();
                if amount <= 0 {
                    is_valid = false;
                } 
                if amount > user_balance {
                    is_valid = false;
                }
            }
        } 
        
        if !is_valid {
            if is_focus {
                textarea.set_style(Style::default().fg(Color::LightRed));
            } else {
                textarea.set_style(Style::default().fg(Color::Red));
            }
            false
        } else {
            if is_focus {
                textarea.set_style(Style::default().fg(Color::LightGreen));
            } else {
                textarea.set_style(Style::default().fg(Color::Green));
            }
            true
        }
    }

    /// Draw the UI.
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let root_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(60),
                    Constraint::Percentage(40),
                ].as_ref()
            )
            .split(f.size());

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ].as_ref()
            )
            .split(root_chunks[0]);
        
        let top_left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ].as_ref()
            )
            .split(top_chunks[0]);
        
        let middle_block = Block::default()
            .title("Create Transaction")
            .borders(Borders::ALL);
        let inner_rect = middle_block.inner(top_chunks[1]);
        let top_middle_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(4),
                ].as_ref()
            )
            .split(inner_rect);

        let top_right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ].as_ref()
            )
            .split(top_chunks[2]);
        
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ].as_ref()
            )
            .split(root_chunks[1]);

        

        let status_text_transform = | status: &BTreeMap<String, String>| {
            let mut status_vec: Vec<String> = vec![];
            for status_item in status {
                status_vec.push(format!("{:?}", status_item));
            }
            status_vec.join("\n")
        };

        let bordered_block_gen = |title| {
            Block::default()
                .title(title)
                .borders(Borders::ALL)
        };


        let paragraph_gen = |text, title, is_wrap| {
            let para = Paragraph::new(text)
                //.style(Style::default().bg(Color::White).fg(Color::Black))
                .block(bordered_block_gen(title))
                .alignment(Alignment::Left);
            if is_wrap {para.wrap(Wrap {trim: true})}
            else {para}
        };

        let logger_gen = |textvec: &Vec<String>, count: usize, title, is_wrap| {
            let mut head_reversed = textvec[textvec.len() - std::cmp::min(count, textvec.len())..textvec.len()].to_vec();
            head_reversed.reverse();
            paragraph_gen(head_reversed.join("\n"), title, is_wrap)
        };

        f.render_widget(paragraph_gen(status_text_transform(&self.blocktree_status), "BlockTree Status", false), top_left_chunks[0]);
        f.render_widget(paragraph_gen(status_text_transform(&self.network_status), "Network Status", false), top_left_chunks[1]);
        f.render_widget(paragraph_gen(status_text_transform(&self.txpool_status), "TxPool Status", false), top_right_chunks[0]);
        f.render_widget(paragraph_gen(status_text_transform(&self.miner_status), "Miner Status", false), top_right_chunks[1]);
        f.render_widget(logger_gen(&self.notify_log, 20, "Notify Log", true), bottom_chunks[0]);
        f.render_widget(logger_gen(&self.stderr_log, 20, "STDERR Log", true), bottom_chunks[1]);

        f.render_widget(middle_block, top_chunks[1]);
        let is_sender_valid = App::validate_id_textarea(&mut self.textareas[0], self.textarea_choosing_idx == 0);
        let is_receiver_valid = App::validate_id_textarea(&mut self.textareas[1], self.textarea_choosing_idx == 1);
        let is_message_valid = App::validate_message_textarea(&mut self.textareas[2], self.textarea_choosing_idx == 2, self.user_balance);
        self.are_inputs_valid = is_sender_valid && is_receiver_valid && is_message_valid;
        f.render_widget(
            Paragraph::new(format!("Balance: ${}", self.user_balance))
                .alignment(Alignment::Left).style(Style::default().fg(Color::LightYellow)),
            top_middle_chunks[0]);
        f.render_widget(self.textareas[0].widget(), top_middle_chunks[1]);
        f.render_widget(self.textareas[1].widget(), top_middle_chunks[2]);
        f.render_widget(self.textareas[2].widget(), top_middle_chunks[3]);
        f.render_widget(
            Paragraph::new("Press Up/Down to change input box\nPress ENTER to create transaction".to_string())
                .alignment(Alignment::Left).style(Style::default().fg(Color::LightBlue)), 
            top_middle_chunks[4]);
     }
}

