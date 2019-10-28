extern crate clap;
extern crate serde_derive;
extern crate ramhorns;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::vec::Vec;

use clap::{App, Arg, SubCommand};
use ramhorns::{Template, Content};
use serde_derive::{Deserialize, Serialize};
use toml::Value;

#[derive(Deserialize, Serialize, Debug)]
struct Profile {
    #[serde(skip)]
    name: String,
    author: String,
    username: String,
    email: String,
    url: Option<String>,
}

#[derive(Content)]
struct UrlRenderData {
    username: String,
    project: String,
}

impl Profile {

    fn render_data(&self, project: String) -> UrlRenderData {
        UrlRenderData{
            project: project,
            username: self.username.clone()
        }
    }

}

struct GitProfilesApp<'a> {
    profiles: Option<Vec<Profile>>,
    args: Option<clap::ArgMatches<'a>>,
}

impl GitProfilesApp<'_> {

    fn new<'a>() -> Result<GitProfilesApp<'a>, std::io::Error> {
        let mut app = GitProfilesApp{ profiles: None, args: None };
        app.parse_args();

        let profiles = app.load_profiles()?;
        app.profiles = Some(profiles);
        Ok(app)
    }

    fn parse_args(&mut self) {
        self.args = Some(App::new("git-profile")
                .version("1.0")
                .author("David Futcher <david@futcher.io>")
                .about("Easy profiles for git")
                .subcommand(
                    SubCommand::with_name("new")
                        .about("Create new profile")
                )
                .subcommand(
                    SubCommand::with_name("list")
                        .about("List profiles"))
                .subcommand(
                    SubCommand::with_name("use")
                        .about("Switch profile")
                        .arg(Arg::with_name("PROFILE")
                                .help("Profile to operate on")
                                .required(true)
                                .takes_value(true))
                )
                .subcommand(
                    SubCommand::with_name("url")
                        .about("Generate remote url")
                        .arg(Arg::with_name("PROJECT")
                                .help("Project name")
                                .required(true))
                        .arg(Arg::with_name("PROFILE")
                                .short("p")
                                .long("profile")
                                .takes_value(true)
                                .help("Profile to use"))
                )
                .subcommand(
                    SubCommand::with_name("author")
                        .about("Get profile's author string in git format")
                        .arg(Arg::with_name("PROFILE")
                                .short("p")
                                .long("profile")
                                .help("Profile to use")
                                .takes_value(true))
                )
                .get_matches());
    }

    fn load_profiles(&mut self) -> Result<Vec<Profile>, std::io::Error> {
        let mut profiles = Vec::new();
        // TODO: Use home-dir here ... also handle first-run no file etc.
        let mut file = File::open(".git_profiles")?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        let data_tables = match toml::from_str(&contents)? {
            Value::Table(table) => table.into_iter().collect(),
            _ => HashMap::new(),
        };

        for (key, value) in data_tables {
            let mut profile: Profile = value.try_into()?;
            profile.name = key;
            profiles.push(profile);
        }

        return Ok(profiles);
    }

    fn get_profile(&self, profile_name: String) -> Option<&Profile> {
        if let Some(profiles) = &self.profiles {
            return profiles.iter().find(|p| p.name == profile_name)
        }

        None
    }

    fn get_profile_by_email(&self, email: String) -> Option<&Profile> {
        if let Some(profiles) = &self.profiles {
            return profiles.iter().find(|p| p.email == email);
        }

        None
    }

    fn get_default_profile(&self) -> Option<&Profile> {
        // TODO: Need to handle the case this is run in a non-gitified dir
        let email = git_command(vec!["config", "user.email"]);
        if let Some(profile) = self.get_profile_by_email(email) {
            return Some(profile);
        }

        if let Some(profiles) = &self.profiles {
            if profiles.len() > 0 {
                return Some(&self.profiles.as_ref().unwrap()[0]);
            }
        }

        None
    }

    fn handle_list(&self) {
        if let Some(profiles) = &self.profiles {
            for profile in profiles {
                println!("{}", profile.name);
            }
        }
    }

    fn handle_use(&self, target: String) {
        let profile = self.get_profile(target)
            .expect("Invalid profile name"); // TODO: handle error case

        // TODO: These have results
        git_command(vec!["config", "user.name", profile.author.as_ref()]);
        git_command(vec!["config", "user.email", profile.email.as_ref()]);
    }

    fn handle_url(&self, project_name: String, profile_name: Option<String>) {
        let profile_opt = match profile_name {
            Some(name) => self.get_profile(name),
            None => self.get_default_profile()
        };

        match profile_opt {
            None => {
                println!("{:?}", self.profiles);
                println!("Couldn't find specified profile, or work out a default");
            },
            Some(profile) => {
                let urlspec = match &profile.url {
                    Some(url) => url.as_ref(),
                    None => "git@github.com:{{username}}/{{project}}"
                };

                let template = Template::new(urlspec).unwrap(); // TODO: danger unwrap
                println!("{}", template.render(&profile.render_data(project_name.to_string())));
            }
        };
    }

    fn handle_author(&self, profile_name: Option<String>) {
        let profile_opt = match profile_name {
            Some(name) => self.get_profile(name),
            None => self.get_default_profile()
        };

        match profile_opt {
            None => {
                println!("{:?}", self.profiles);
                println!("Couldn't find specified profile, or work out a default");
            },
            Some(profile) => {
                println!("{} <{}>", profile.author, profile.email);
            }
        }
    }
}

fn git_command(args: Vec<&str>) -> String {
    let mut command = Command::new("git");

    for arg in args {
        command.arg(arg);
    }

    let output_streams = command.output().expect("failed to execute process");
    let output = std::str::from_utf8(&output_streams.stdout).unwrap().trim_end();

    return output.to_string();
}

fn main() {
    let app = GitProfilesApp::new().expect("failed to initialise");

    if let Some(args) = &app.args {
        match args.subcommand() {
            ("list", _) => app.handle_list(),
            ("new", _) => println!("New!"),
            ("use", Some(sub_matches)) => {
                let profile = sub_matches.value_of("PROFILE").unwrap();
                app.handle_use(profile.to_string());
            },
            ("url", Some(sub_matches)) => {
                let project_name = sub_matches.value_of("PROJECT").unwrap();
                let profile_name = sub_matches.value_of("PROFILE").map(|x| x.to_string());
                app.handle_url(project_name.to_string(), profile_name);
            },
            ("author", Some(sub_matches)) => {
                let profile = sub_matches.value_of("PROFILE").map(|x| x.to_string());
                app.handle_author(profile);
            }
            _ => println!("other"),
        };
    }

}
