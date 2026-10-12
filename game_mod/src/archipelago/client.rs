use std::{
  collections::HashSet,
  io::{self, Write},
  time::Duration,
};

use archipelago_rs::{self as ap_rs, ConnectionOptions, ConnectionState};
use ustr::Ustr;
use windows::Win32::System::Console::{AllocConsole, FreeConsole};

use crate::archipelago::slot::SlotData;

pub struct Archipelago {
  connection: ap_rs::Connection<SlotData>,
}

impl Archipelago {
  fn connect<S, N, P>(url: S, slot: N, pass: Option<P>) -> ap_rs::Connection<SlotData>
  where
    S: Into<String>,
    N: Into<Ustr>,
    P: Into<String>,
  {
    let mut options = ConnectionOptions::default().tags(["DEATH_LINK"]);
    if let Some(pass) = pass {
      options = options.password(pass);
    }
    ap_rs::Connection::new(url, slot, "Luftrausers".into(), options)
  }

  pub fn with_console_prompt() -> Self {
    // Display a windows console for connection detail. Maybe later I'll make it
    // look nicer and more in the game

    // apparently you can't break with a return value in a for or while loop
    // guess we choose an error prone, statically unanalyzable way to do it instead
    let connection = loop {
      let (server, slot, password) = Self::console_prompt();
      let password = if password.is_empty() {
        None
      } else {
        Some(password)
      };
      let conn = Self::connect(server, slot, password);
      println!("Connecting...");

      let mut retries = 30;
      while conn.is_connecting() && retries > 0 {
        std::thread::sleep(Duration::from_millis(100));
        retries -= 1;
      }

      match conn.state() {
        ConnectionState::Disconnected(err) => {
          println!("Failed to connect: {}\n\n", err);
          continue;
        }
        ConnectionState::Connecting(_) => {
          println!("Failed to connect: Timeout\n\n");
          continue;
        }
        ConnectionState::Connected(_) => break conn,
      };
    };

    unsafe {
      let _ = FreeConsole();
    }

    Self { connection }
  }

  fn console_prompt() -> (String, String, String) {
    unsafe {
      let _ = AllocConsole();
    }

    println!("=== Luftrausers Archipelago Setup ===");

    print!("Server (defaults to archipelago.gg:38281): ");
    let mut server = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut server).unwrap();
    if server.is_empty() {
      server = String::from("archipelago.gg:38281");
    }
    let server = server.trim().to_string();
    println!("Got server name: {}", server);

    let slot = loop {
      let mut slot = String::new();
      print!("Slot Name: ");
      io::stdout().flush().unwrap();
      io::stdin().read_line(&mut slot).unwrap();
      let slot = slot.trim().to_string();

      if !slot.is_empty() {
        break slot;
      }
    };
    println!("Got slot name: {}", slot);

    print!("Password (press enter if none): ");
    let mut password = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim().to_string();

    (server, slot, password)
  }

  pub fn client(&mut self) -> &mut ap_rs::Client<SlotData> {
    if let ConnectionState::Connected(client) = self.connection.state_mut() {
      client
    } else {
      // in theory this should never happen (assuming we connect the first time),
      // but I could be reading the crate docs wrong
      unreachable!();
    }
  }

  pub fn update(&mut self) {
    let events = self.connection.update();

    //for e in events {
    //  match e {
    //    Event::ReceivedItems(id) => self.items,
    //  }
    //}
  }
}
