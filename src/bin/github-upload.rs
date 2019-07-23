use fs_extra;
use git2;
use reqwest::{self, header};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::Path,
};
use tempfile;
use toml;

#[derive(Deserialize)]
struct Config {
    title: String,
    github: GitHubConfig,
}

#[derive(Deserialize)]
struct GitHubConfig {
    username: String,
    repository: String,
    domain: Option<String>,
    author: Option<String>,
    email: Option<String>,
}

static PROMPT: &str = r#"
This application wants publish your comic to GitHub. If you don't have
a GitHub account, you'll need to create one first. To allow this
application to upload your files and set up website hosting,
go to https://github.com/settings/tokens and generate an auth token
with the "public_repo" permission.
"#;

fn read_token(prompt: &str, path: impl AsRef<Path>) -> io::Result<String> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    fs::write(&path, &buffer)?;
    buffer = buffer.trim().into();
    if buffer.is_empty() {
        read_token(prompt, path)
    } else {
        Ok(buffer)
    }
}

const API_V3: &str = "application/vnd.github.v3+json";

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => {
            println!("{}", err);
            println!("Press [enter] to finish.");
            io::stdin().read_line(&mut String::new()).unwrap();
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let config_text = fs::read_to_string("input/config.toml")?;
    let Config {
        title,
        github: config,
    } = toml::from_str::<Config>(&config_text)?;

    let token_path = Path::new("auth-token.txt");
    let token = if token_path.is_file() {
        fs::read_to_string(token_path)?
    } else {
        println!("{}", PROMPT);
        read_token("Token", token_path)?
    };
    let token = token.trim();

    let client = reqwest::Client::new();
    let res = client
        .get(&format!(
            "https://api.github.com/repos/{}/{}",
            &config.username, &config.repository,
        ))
        .header(header::ACCEPT, API_V3)
        .basic_auth(&config.username, Some(&token))
        .send()?;

    let description = format!(
        "{}, published with comic-publisher, do not edit manually.",
        &title
    );
    let mut res = if res.status().is_success() {
        res
    } else {
        println!("Creating repository...");
        #[derive(Serialize)]
        struct MakeRepo<'a> {
            name: &'a str,
            description: &'a str,
        }
        client
            .post("https://api.github.com/user/repos")
            .header(header::ACCEPT, API_V3)
            .basic_auth(&config.username, Some(&token))
            .json(&MakeRepo {
                name: &config.repository,
                description: &description,
            })
            .send()?
    };

    #[derive(Deserialize, Debug)]
    struct Repo {
        description: Option<String>,
        html_url: String,
        size: usize,
    }
    let repo: Repo = res.json()?;

    if repo.size != 0 && repo.description.as_ref() != Some(&description) {
        // @TODO: Handle keeping the message open.
        println!(
            r#"The repository specified ({}) has an unexpected description.
   Found: {}
Expected: {}
You may have accidentally included the wrong repository name. If you're sure this is the correct repository, visit {} and change the description to match.
"#,
            &config.repository,
            &repo.description.unwrap_or_else(String::new),
            &description,
            &repo.html_url,
        );
        Err("Error")?;
    }

    let temp_dir = tempfile::tempdir()?;
    let mut sources = Vec::new();
    for dir in fs::read_dir("output")? {
        sources.push(dir?.path());
    }
    println!("Copying directories...");
    fs_extra::copy_items(
        &sources,
        &temp_dir.path(),
        &fs_extra::dir::CopyOptions::new(),
    )?;
    if let Some(ref domain) = config.domain {
        println!("Adding CNAME...");
        fs::write(temp_dir.path().join("CNAME"), domain)?;
    }
    println!("Creating git repository...");
    let repository = git2::Repository::init(&temp_dir.path())?;
    let mut index = repository.index()?;
    println!("Adding files...");
    index.add_all::<_, &[&str]>(&[], git2::IndexAddOption::DEFAULT, None)?;
    println!("Writing...");
    let oid = index.write_tree()?;
    let tree = repository.find_tree(oid)?;
    println!("Committing...");
    repository.commit(
        Some("HEAD"),
        &git2::Signature::now(
            &config.author.unwrap_or_default(),
            &config.email.unwrap_or_default(),
        )?,
        &git2::Signature::now("comic-publisher", "code+comic-publisher@witchoflight.com")?,
        "Comic upload",
        &tree,
        &[],
    )?;
    let url = format!(
        "https://{}:{}@github.com/{}/{}.git",
        &config.username, &token, &config.username, &config.repository
    );
    let mut remote = repository.remote("github", &url)?;
    println!("Pushing...");
    remote.push(&["+refs/heads/master:refs/heads/master"], None)?;

    Ok(())
}
