use std::fs;
use std::io;
use std::io::Read;

use anyhow::Result;
use async_graphql::ServerError;
use clap::Parser;
use inquire::Confirm;
use log::Level;
use resource::resource_str;
use stripmargin::StripMargin;

use super::command::{Cli, Command};
use crate::async_graphql_hyper;
use crate::blueprint::Blueprint;
use crate::cli::fmt::Fmt;
use crate::config::Config;
use crate::http::start_server;
use crate::print_schema;
use crate::valid::ValidationError;

pub async fn run() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Command::Start { file_path, log_level } => {
      env_logger::Builder::new()
        .filter_level(log_level.unwrap_or(Level::Info).to_level_filter())
        .init();
      let config = Config::from_file_paths(file_path.iter()).await?;
      start_server(config).await?;
      Ok(())
    }
    Command::Check { file_path, n_plus_one_queries, schema, operations } => {
      // validate configurations
      let config = Config::from_file_paths(file_path.iter()).await?;
      let blueprint = Blueprint::try_from(&config);
      match blueprint {
        Ok(blueprint) => {
          // validate operations
          let gql_schema = blueprint.to_schema();
          match operations {
            Some(operations) => {
              for o in &operations {
                let request: async_graphql_hyper::GraphQLRequest = make_request_from_file(o)?;
                let result = gql_schema.validate(request.0).await;
                if let Err(e) = result {
                  return Err(<Vec<ServerError> as std::convert::Into<ValidationError<String>>>::into(e).into());
                } else {
                  return Result::<()>::Ok(());
                }
              }
            }
            // Don't check
            _ => {}
          }
          display_config(&config, n_plus_one_queries);
          if schema {
            display_schema(&blueprint);
          }
          Ok(())
        }
        Err(e) => Err(e.into()),
      }
    }
    Command::Init { file_path } => Ok(init(&file_path).await?),
  }
}

pub async fn init(file_path: &str) -> Result<()> {
  let tailcallrc: resource::Resource<str> = resource_str!("examples/.tailcallrc.graphql");

  let ans = Confirm::new("Do you want to add a file to the project?")
    .with_default(false)
    .prompt();

  match ans {
    Ok(true) => {
      let file_name = inquire::Text::new("Enter the file name:")
        .with_default(".graphql")
        .prompt()
        .unwrap_or_else(|_| String::from(".graphql"));

      let file_name = format!("{}.graphql", file_name.strip_suffix(".graphql").unwrap_or(&file_name));

      let confirm = Confirm::new(&format!("Do you want to create the file {}?", file_name))
        .with_default(false)
        .prompt();

      match confirm {
        Ok(true) => {
          fs::write(format!("{}/{}", file_path, &file_name), "")?;

          let graphqlrc = format!(
            r#"|schema:
               |- './{}'
               |- './.tailcallrc.graphql'
          "#,
            &file_name
          )
          .strip_margin();
          fs::write(format!("{}/.graphqlrc.yml", file_path), graphqlrc)?;
        }
        Ok(false) => (),
        Err(e) => return Err(e.into()),
      }
    }
    Ok(false) => (),
    Err(e) => return Err(e.into()),
  }

  fs::write(
    format!("{}/.tailcallrc.graphql", file_path),
    tailcallrc.as_ref().as_bytes(),
  )?;
  Ok(())
}

pub fn display_schema(blueprint: &Blueprint) {
  Fmt::display(Fmt::heading(&"GraphQL Schema:\n".to_string()));
  let sdl = blueprint.to_schema();
  Fmt::display(print_schema::print_schema(sdl));
}

fn display_config(config: &Config, n_plus_one_queries: bool) {
  Fmt::display(Fmt::success(&"No errors found".to_string()));
  let seq = vec![Fmt::n_plus_one_data(n_plus_one_queries, config)];
  Fmt::display(Fmt::table(seq));
}

fn make_request_from_file(file_path: &str) -> Result<async_graphql_hyper::GraphQLRequest, anyhow::Error> {
  let f = fs::File::open(file_path)?;
  let mut reader = io::BufReader::new(f);
  let mut buffer = Vec::new();

  // Read file into vector.
  reader.read_to_end(&mut buffer)?;
  let request = async_graphql_hyper::GraphQLRequest(async_graphql::Request::new(String::from_utf8(buffer)?));
  return Ok(request);
}

impl Into<ValidationError<String>> for Vec<ServerError> {
  fn into(self) -> ValidationError<String> {
    let mut err = ValidationError::<String>::empty();
    for se in self {
      err = err.append(format!("{:?}", se));
    }
    return err;
  }
}
