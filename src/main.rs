use std::env::args_os;
use std::process::exit;

use crate::command_parser::CommandMatchParser;
use crate::output_handler::{handle_error, handle_output};
// use clap::AppSettings::{ArgRequiredElseHelp, SubcommandRequiredElseHelp};
use clap::{load_yaml, App};
use sap_adt_bindings::net::SAPClient;

pub mod command_parser;
pub mod output_handler;

use sap_adt_bindings::app_config::AppConfig;

#[tokio::main]
async fn main() {
    let cli_yaml = load_yaml!("cli.yaml");
    let mut app = App::from(cli_yaml);

    // Check manually if no argument was given because clap throws exit code 2
    if args_os().count() == 1 {
        app.print_help().expect("Could not print help");
        exit(0);
    }

    let matches = app.get_matches();

    let mut config = CommandMatchParser::parse(&matches);

    let mut app_conf = AppConfig::init();
    let mut client: SAPClient;

    let dest = app_conf.get_default_destination();
    println!("{:?}", dest);
    let update_session_file: bool;

    if let Some(session) = app_conf.get_session_from_sys(&dest.sys_id) {
        client = SAPClient::from_session(&dest, session);
        update_session_file = false;
    } else {
        client = SAPClient::new(&dest);
        update_session_file = true;
    }

    match config.send_with(&mut client).await {
        Ok(()) => handle_output(config.get_response().unwrap()),
        Err(e) => handle_error(e),
    }

    if update_session_file {
        app_conf.set_session_for_sys("ITK", &client.get_session().unwrap());
    }
    app_conf.update_file();
}
