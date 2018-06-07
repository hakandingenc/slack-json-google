extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate url;
#[macro_use]
extern crate lazy_static;

pub mod hostmap;
pub mod server;

use futures::{Stream, future::Future};
use hostmap::HostMap;
use hyper::server::Http;
use regex::Regex;
use server::Server;
use std::{thread, io::{self, BufReader, prelude::*}, path::Path,
          sync::{Arc, Mutex, mpsc::{self, Receiver, Sender}}};

lazy_static! {
    static ref RE_COMMAND: Regex =
        Regex::new(r#"^(add|rm|ls)(?: "(\w+)")?(?: "([a-zA-Z0-9:/.]+)")?$"#).unwrap();
}

pub enum Command {
    Add(String, String),
    Remove(String),
    List,
}

pub fn start_server(
    addr: std::net::SocketAddr,
    dict_file: &'static Path,
    response_to_slack: &str,
) -> hyper::Result<()> {
    let (send_callback_id, recv_callback_id): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (send_url, recv_url): (Sender<Option<String>>, Receiver<Option<String>>) = mpsc::channel();
    let recv_url_mutex = Arc::new(Mutex::new(recv_url));
    thread::spawn(move || {
        let dict_file = dict_file.clone();
        let (send_new_line, recv_new_line): (Sender<Command>, Receiver<Command>) = mpsc::channel();
        thread::spawn(move || {
            println!("Before");
            BufReader::new(io::stdin()).lines().for_each(|new_line| {
                println!("After");
                let new_line = new_line.expect("Could not read line");
                let re_try = RE_COMMAND.captures(&new_line);
                println!("the matching result of {}: {:?}", new_line, re_try);
                match re_try {
                    Some(array) => {
                        println!("array[0] is {}", &array[0]);
                        match &array[0] {
                            "add" => {
                                send_new_line
                                    .send(Command::Add(array[1].to_string(), array[2].to_string()));
                            }
                            "rm" => {
                                send_new_line.send(Command::Remove(array[1].to_string()));
                            }
                            "ls" => {
                                send_new_line.send(Command::List);
                            }
                            _ => unreachable!(),
                        }
                    }
                    None => {
                        println!("Command not recognized");
                    }
                }
            });
        });
        let mut dictionary = HostMap::new_from_file(dict_file).unwrap();
        loop {
            if let Ok(new_command) = recv_new_line.try_recv() {
                match new_command {
                    Command::List => {
                        println!("List of mappings:\n{:#?}", dictionary);
                    }
                    Command::Add(callback_id, url) => {
                        println!("Mapping for {} has been inserted", callback_id);
                        dictionary.insert(callback_id, url);
                    }
                    Command::Remove(callback_id) => match dictionary.remove(&callback_id) {
                        Some(_) => {
                            println!("Mapping for {} has been removed", callback_id);
                        }
                        None => {
                            println!("Can't remove {} because it doesn't exist", callback_id);
                        }
                    },
                }
            }
            let callback_id = recv_callback_id.recv().unwrap();
            let url = dictionary.resolve_callback(&callback_id);
            send_url.send(url);
        }
    });

    let mut core = tokio_core::reactor::Core::new()?;
    let server_handle = core.handle();
    let client_handle = core.handle();
    let response_to_slack = response_to_slack.to_string();
    let serve = Http::new().serve_addr_handle(&addr, &server_handle, move || {
        Ok(Server::new(
            client_handle.clone(),
            response_to_slack.clone(),
            send_callback_id.clone(),
            recv_url_mutex.clone(),
        ))
    })?;

    println!(
        "Listening on http://{} with 1 thread.",
        serve.incoming_ref().local_addr()
    );
    let h2 = server_handle.clone();
    server_handle.spawn(
        serve
            .for_each(move |conn| {
                h2.spawn(
                    conn.map(|_| ())
                        .map_err(|err| println!("serve error: {:?}", err)),
                );
                Ok(())
            })
            .map_err(|_| ()),
    );
    core.run(futures::future::empty::<(), hyper::Error>())
}

fn print_carets() -> io::Result<()> {
    print!(">>> ");
    io::stdout().flush()
}
