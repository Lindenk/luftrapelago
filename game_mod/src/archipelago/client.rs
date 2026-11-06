use std::{
  io::{self, Write},
  time::Duration,
};

use archipelago_rs::{self as ap_rs, ConnectionOptions, ConnectionState};
use ustr::Ustr;
use windows::Win32::System::Console::{AllocConsole, FreeConsole};

use crate::{
  archipelago::{items::APItem, slot::SlotData},
  game_data::plane::Part,
};

pub struct APClient {
  connection: ap_rs::Connection<SlotData>,
}

impl APClient {
  fn connect<S, N, P>(url: S, slot: N, pass: Option<P>) -> ap_rs::Connection<SlotData>
  where
    S: Into<String>,
    N: Into<Ustr>,
    P: Into<String>,
  {
    let mut options = ConnectionOptions::default()
      .tags(["DEATH_LINK"])
      .receive_items(archipelago_rs::ItemHandling::OtherWorlds {
        own_world: true,
        starting_inventory: true,
      });
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
    let mut connection = loop {
      //let (server, slot, password) = Self::console_prompt();
      let (server, slot, password) = (
        "localhost:53087".to_string(),
        "Pineapple_LRS".to_string(),
        "".to_string(),
      );
      let password = if password.is_empty() {
        None
      } else {
        Some(password)
      };
      let mut conn = Self::connect(server, slot, password);
      println!("Connecting...");

      let mut retries = 100;
      while conn.is_connecting() && retries > 0 {
        conn.update();
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
    // fetch all outstanding items before we start
    while connection.client_mut().unwrap().sync().is_err() {
      std::thread::sleep(Duration::from_millis(20));
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

  // utility functions
  pub fn get_items(&self) -> impl Iterator<Item = APItem> {
    self
      .connection
      .client()
      .unwrap()
      .received_items()
      .iter()
      .filter_map(|i| APItem::try_from(i.item().id()).ok())
  }
  pub fn get_parts(&self) -> impl Iterator<Item = Part> {
    self.get_items().filter_map(|i| Part::try_from(i).ok())
  }
  pub fn has_enough_rank_keys(&self, mission_num: u8) -> bool {
    if let Some(mission_lock) = self
      .connection
      .client()
      .unwrap()
      .slot_data()
      .rank_key_mission_lock
    {
      let num_keys = self
        .get_items()
        .filter(|i| matches!(i, APItem::ProgressiveRank))
        .count() as u8;

      // since mission_num starts at 0, we'd be off by 1
      mission_num < mission_lock + num_keys - 1
    } else {
      true // rank keys are disabled
    }
  }
}
