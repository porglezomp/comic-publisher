use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};
use tera::Tera;
use toml;

#[derive(Deserialize, Serialize)]
struct Config {
    title: String,
    comics: Vec<Comic>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Comic {
    folder: PathBuf,
    title: String,
    description: String,
}

fn main() -> io::Result<()> {
    let tera = match Tera::new("templates/**/*") {
        Ok(tera) => tera,
        Err(err) => {
            println!("Parsing error(s): {}", err);
            ::std::process::exit(1);
        }
    };
    let root = Path::new("input");
    if !root.is_dir() {
        fs::create_dir("input")?;
        let mut config = File::create("input/config.toml")?;
        config.write_all(CONFIG.as_bytes())?;
        let mut readme = File::create("input/README.txt")?;
        readme.write_all(README.as_bytes())?;
        fs::create_dir("input/comic")?;
        return Ok(());
    }

    fs::create_dir_all("output")?;
    let config_text = fs::read_to_string("input/config.toml")?;
    let config: Config = toml::de::from_str(&config_text)
        .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;
    let mut errors = Vec::new();
    let mut comics = Vec::new();
    for comic in config.comics {
        let comic_folder = root.join(&comic.folder);
        if !comic_folder.is_dir() {
            errors.push(format!(
                "Comic folder {:?} is not a directory",
                comic_folder
            ));
            continue;
        }
        let mut pages = Vec::new();
        for page in fs::read_dir(comic_folder)? {
            match page {
                Ok(page) => pages.push(page.path()),
                Err(err) => errors.push(format!("Error reading page {}", err)),
            }
        }
        comics.push((comic, pages));
    }

    let mut context = tera::Context::new();
    context.insert("comics", &comics);
    context.insert("title", &config.title);

    let result = tera
        .render("index.html", context)
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Couldn't render index: {}", e)))?;
    fs::write("output/index.html", result)?;

    // @TODO: Produce the individual pages

    if !errors.is_empty() {
        let mut error_buf = File::create("errors.txt")?;
        for error in errors {
            let _ = writeln!(error_buf, "{}", error);
        }
        Err(io::Error::new(ErrorKind::Other, "Some errors occurred."))
    } else {
        Ok(())
    }
}

static CONFIG: &str = r#"title = "Comic Website"

[[comics]]
folder = "comic"
title = "First Comic"
description = """
This is just an example description.
You can write them on multiple lines like this if you use 3 quotes like this.
"""
"#;

static README: &str = r#"How to use this tool.

Make a folder for each comic inside this "input" folder, with the pages
inside. Put your pages inside the folder and name your pages in
alphabetical order. An easy way to do this is just number them all,
like 00-firstpage.png for example.

Edit the config.toml file. You should be able to use any text editor on
your computer. You want to make one entry for each comic. They will be
listed on the site in the order you put them here. Here's an example
that has multiple comics listed:

    title = "A Comics Site"

    [[comics]]
    folder = "comic"
    title = "First Comic"
    description = """
    This is the description for the first comic.
    You can write it with multiple lines if you have 3 quotes like that.
    """

    [[comics]]
    folder = "comic2"
    title = "Second Comic"
    description = """
    This example isn't in the default.
    """

Once this is set up, every time your run the program, it will build
your comic into the output folder.
"#;
