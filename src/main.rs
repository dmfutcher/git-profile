extern crate clap;
extern crate dirs;
extern crate serde_derive;
extern crate ramhorns;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::from_utf8;
use std::vec::Vec;

use clap::{App, Arg, SubCommand};
use ramhorns::{Template, Content};
use serde_derive::{Deserialize, Serialize};
use toml::Value;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Profile {
    #[serde(skip)]
    name: String,
    author: String,
    email: String,
    username: Option<String>,
    url: Option<String>,
}

#[derive(Content)]
struct UrlRenderData {
    username: String,
    project: String,
}

impl Profile {

    fn new(profile_name: &str, author_name: &str, author_email: &str) -> Profile {
        Profile{
            name: profile_name.to_owned(),
            author: author_name.to_owned(),
            email: author_email.to_owned(),
            username: None,
            url: None,
        }
    }

    fn with_remote_url(&mut self, remote: &str) -> &mut Profile {
        self.url = Some(remote.to_owned());
        self
    }

    fn with_username(&mut self, user: &str) -> &mut Profile {
        self.username = Some(user.to_owned());
        self
    }

    fn as_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        map.insert("author".to_owned(), self.author.clone());
        map.insert("email".to_owned(), self.email.clone());

        if let Some(username) = &self.username {
            map.insert("username".to_owned(), username.clone());
        }

        if let Some(url) = &self.url {
            map.insert("url".to_owned(), url.clone());
        }

        map
    }

    fn render_data(&self, project: String) -> UrlRenderData {
        let username = match &self.username {
            Some(user) => user.to_owned(),
            _ => String::new()
        };

        UrlRenderData{
            project: project,
            username: username
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
                .version("0.1")
                .author("David Futcher <david@futcher.io>")
                .about("Easy multi-identity profiles for git")
                .subcommand(
                    SubCommand::with_name("new")
                        .about("Create new profile")
                        .arg(Arg::with_name("PROFILE")
                                .help("Name of profile to create")
                                .required(true))
                        .arg(Arg::with_name("AUTHOR")
                                .required(true))
                        .arg(Arg::with_name("EMAIL")
                                .required(true))
                        .arg(Arg::with_name("USERNAME")
                                .short("u")
                                .long("username")
                                .takes_value(true))
                        .arg(Arg::with_name("URL")
                                .short("r")
                                .long("remote")
                                .takes_value(true))
                        // TODO: Add --edit arg, opens file in editor _after_ writing new profile data
                )
                .subcommand(
                    App::new("list")
                        .alias("ls")
                        .about("List profiles"))
                .subcommand(
                    SubCommand::with_name("use")
                        .about("Switch profile")
                        .arg(Arg::with_name("PROFILE")
                                .help("Profile to operate on")
                                .required(true)
                                .takes_value(true))
                        // TODO: Add --global flag, operating on git config --global
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
                .subcommand(
                    SubCommand::with_name("edit")
                        .about("Edit profiles")  
                        .arg(Arg::with_name("EDITOR")
                                .long("editor")
                                .takes_value(true))
                )
                .get_matches());
    }

    fn profiles_file_path(&self) -> Option<PathBuf> {
        match dirs::home_dir() {
            Some(mut path) => {
                path.push(".git_profiles");
                Some(path)
            },
            _ => None
        }
    }

    fn parse_profiles(&self, contents: String) -> Result<Vec<Profile>, std::io::Error> {
        let data_tables = match toml::from_str(&contents)? {
            Value::Table(table) => table.into_iter().collect(),
            _ => HashMap::new(),
        };

        let mut profiles = Vec::new();

        for (key, value) in data_tables {
            let mut profile: Profile = value.try_into()?;
            profile.name = key;
            profiles.push(profile);
        }

        Ok(profiles)
    }

    fn load_profiles(&mut self) -> Result<Vec<Profile>, std::io::Error> {
        let path_buf = self.profiles_file_path().expect("expected valid profile file-path");
        let path = path_buf.as_path();
       
        if Path::exists(path) {
            let mut file = fs::File::open(path)?;
            let mut contents = String::new();

            file.read_to_string(&mut contents)?;
            return self.parse_profiles(contents);
        }

        Ok(vec![])
    }

    fn save_profiles(&self, profiles: Vec<&Profile>) -> Result<(), Box<dyn Error>> {
        let path_buf = self.profiles_file_path().expect("expected valid profile file-path");
        let path = path_buf.as_path();

        if !Path::exists(path) {
            fs::File::create(path)?;
        }

        let mut tables: HashMap<String, toml::Value> = HashMap::new();
        for profile in profiles {
            let key = profile.name.clone();
            let table = toml::Value::try_from(profile.as_map())?;
            tables.insert(key, table);
        }

        let data = toml::to_string_pretty(&tables)?;
        fs::write(path, data.as_bytes())?;

        Ok(())
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

    fn get_profile_in_local_use(&self) -> Option<&Profile> {
        // TODO: Need to handle the case this is run in a non-git dir. Manually detect ./.git dir?
        let email = git_command(vec!["config", "user.email"]);
        if let Some(profile) = self.get_profile_by_email(email) {
            return Some(profile);
        }

        None
    }

    fn get_default_profile(&self) -> Option<&Profile> {
        if let Some(profile) = self.get_profile_in_local_use() {
            return Some(profile)
        }

        if let Some(profiles) = &self.profiles {
            if profiles.len() > 0 {
                return Some(&self.profiles.as_ref().unwrap()[0]);
            }
        }

        None
    }

    /// Unwraps the profile name, finds a matching profile (or falls back to a reasonable default) then executes the 
    /// closure with the profile as it's argument.
    fn with_profile<F>(&self, name: Option<&str>, f: F) 
        where F: Fn(&Profile) -> ()
    {
        let profile_opt = match name {
            Some(name) => self.get_profile(name.to_owned()),
            None => self.get_default_profile()
        };

        match profile_opt {
            None => {
                println!("Couldn't find specified profile, or work out a default");
            },
            Some(profile) => {
                f(profile);
            }
        }
    }

    fn handle_list(&self) {
        let no_profiles = || println!("No profiles defined");

        if let Some(profiles) = &self.profiles {
            if profiles.len() == 0 {
                no_profiles();
                return
            }

            let local_profile = self.get_profile_in_local_use();

            for profile in profiles {

                print!("{}", profile.name);

                if let Some(local) = local_profile {
                    if local == profile {
                        print!(" *");
                    }
                }

                print!("\n");  // TODO: Does this work cross-platform?
            }
        } else {
            no_profiles();
        }
    }

    fn handle_use(&self, target: String) {
        // We never want to fallback to a default when dealing with 'use' cmd, so we don't use `with_profile`, instead
        // handle profile lookup manually
        let profile = self.get_profile(target).expect("Could not find target profile");

        // TODO: These have results we should probably pay attention to
        git_command(vec!["config", "user.name", profile.author.as_ref()]);
        git_command(vec!["config", "user.email", profile.email.as_ref()]);
    }

    fn handle_url(&self, profile_name: Option<&str>, project_name: String) {
        self.with_profile(profile_name, |p| {
            let urlspec = match &p.url {
                Some(url) => url.as_ref(),
                None => "git@github.com:{{username}}/{{project}}"
            };

            let template = Template::new(urlspec).expect("Failed to create template from urlspec");
            println!("{}", template.render(&p.render_data(project_name.to_owned())));
        });
    }

    fn handle_author(&self, profile_name: Option<&str>) {
        self.with_profile(profile_name, |p| println!("{} <{}>", p.author, p.email));
    }

    fn handle_new(&self, profile_name: &str, author_name: &str, author_email: &str, username: Option<&str>, 
                    remote: Option<&str>) 
    {
        let mut profile = Profile::new(profile_name, author_name, author_email);

        if let Some(user) = username {
            profile.with_username(user);
        }

        if let Some(url) = remote {
            profile.with_remote_url(url);
        }

        let mut new_profiles = Vec::new();
        new_profiles.extend(self.profiles.as_ref().unwrap());
        new_profiles.push(&profile);

        let result = self.save_profiles(new_profiles);
        match result {
            Ok(_) => println!("Profile {} created", profile_name),
            Err(e) => println!("Profile create failed: {}", e)
        };
    }

    fn handle_edit(&self, editor_opt: Option<&str>) {
        let editor: String;
        if let Some(val) = editor_opt {
            editor = val.to_owned();
        } else if let Ok(val) = env::var("EDITOR") {
            editor = val;
        } else {
            // TODO: Better fallback value needed, won't work too nicely with Windows
            editor = "vim".to_owned();
        }

        let path = self.profiles_file_path().expect("Failed to get profiles file path");

        let result = Command::new(editor)
                        .arg(path)
                        .status()
                        .expect("edit command failed");

        if result.success() {
            println!("Edit success!");
        } else {
            println!("Edit failed");
        }
    }
}

fn git_command(args: Vec<&str>) -> String {
    let mut command = Command::new("git");

    for arg in args {
        command.arg(arg);
    }

    let output_streams = command.output().expect("failed to execute process");
    let output = from_utf8(&output_streams.stdout).unwrap().trim_end();

    return output.to_owned();
}

fn main() {
    let app = GitProfilesApp::new().expect("profile loading failed, check your profile config");

    if let Some(args) = &app.args {
        match args.subcommand() {
            ("list", _) => app.handle_list(),
            ("new", Some(sub_matches)) => {
                let profile_name = sub_matches.value_of("PROFILE").expect("failed to parse profile name");
                let author_name = sub_matches.value_of("AUTHOR").expect("failed to parse author name");
                let author_email = sub_matches.value_of("EMAIL").expect("failed to parse author email");
                let url = sub_matches.value_of("URL");
                let username = sub_matches.value_of("USER");

                app.handle_new(profile_name, author_name, author_email, username, url);
            },
            ("use", Some(sub_matches)) => {
                let profile_name = sub_matches.value_of("PROFILE").expect("failed to parse profile name");
                app.handle_use(profile_name.to_owned());
            },
            ("url", Some(sub_matches)) => {
                let project_name = sub_matches.value_of("PROJECT").expect("failed to parse project name");
                let profile_name = sub_matches.value_of("PROFILE");
                app.handle_url(profile_name, project_name.to_owned());
            },
            ("author", Some(sub_matches)) => {
                let profile_name = sub_matches.value_of("PROFILE");
                app.handle_author(profile_name);
            },
            ("edit", Some(sub_matches)) => {
                let editor = sub_matches.value_of("EDITOR");
                app.handle_edit(editor);
            },
            _ => println!("{}", args.usage()), // TODO: Should list sub-commands
        };
    }
}
