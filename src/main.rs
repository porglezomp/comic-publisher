use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};
use tera::Tera;
use toml;

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    title: String,
    comics: Vec<ImportComic>,
    copyright: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ImportComic {
    folder: PathBuf,
    thumbnail: PathBuf,
    title: String,
    description: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Comic {
    title: String,
    thumbnail: String,
    url: String,
    description: String,
    pages: Vec<String>,
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
                Ok(page) => pages.push(page),
                Err(err) => errors.push(format!("Error reading page {}", err)),
            }
        }
        let mut pages: Vec<_> = pages
            .iter()
            .map(|p| {
                Path::new("images")
                    .join(p.path().strip_prefix("input").unwrap())
                    .display()
                    .to_string()
            })
            .collect();
        pages.sort();
        comics.push(Comic {
            title: comic.title,
            thumbnail: Path::new("images")
                .join(comic.thumbnail)
                .display()
                .to_string(),
            url: comic.folder.display().to_string(),
            description: comic.description,
            pages,
        });
    }

    for file in fs::read_dir("static")? {
        let file = match file {
            Ok(file) => file,
            Err(err) => {
                errors.push(format!("Error trying to copy: {}", err));
                continue;
            }
        };
        let from = file.path();
        let to = Path::new("output").join(from.strip_prefix("static").unwrap());
        if let Err(err) = fs::copy(&from, &to) {
            errors.push(format!(
                "Failed to copy {} to {}: {}",
                from.display(),
                to.display(),
                err
            ));
        }
    }

    let mut context = tera::Context::new();
    context.insert("comics", &comics);
    context.insert("title", &config.title);
    context.insert("copyright", &config.copyright);

    let result = tera
        .render("index.html", context)
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Couldn't render index: {}", e)))?;
    fs::write("output/index.html", result)?;

    fn copy(path: &str, errors: &mut Vec<String>) {
        let path = Path::new(path);
        let src = Path::new("input").join(path.strip_prefix("images").unwrap());
        let dst = Path::new("output").join(path);
        let dir = dst.parent().unwrap();
        if let Err(err) = fs::create_dir_all(dir) {
            errors.push(format!(
                "Couldn't create directory {}: {}",
                dir.display(),
                err
            ));
        }
        if let Err(err) = fs::copy(&src, &dst) {
            errors.push(format!(
                "Failed to copy {} to {}: {}",
                src.display(),
                dst.display(),
                err
            ));
        }
    }

    for comic in comics {
        copy(&comic.thumbnail, &mut errors);
        for page in &comic.pages {
            copy(&page, &mut errors);
        }

        let mut context = tera::Context::new();
        context.insert("comic", &comic);
        context.insert("title", &config.title);
        context.insert("copyright", &config.copyright);

        match tera.render("comic.html", context) {
            Ok(result) => {
                let dir = Path::new("output").join(comic.url);
                fs::create_dir_all(&dir)?;
                fs::write(dir.join("index.html"), result)?;
            }
            Err(err) => errors.push(format!("Couldn't render comic {}: {}", &comic.title, err)),
        }
    }

    if !errors.is_empty() {
        let mut error_buf = File::create("errors.txt")?;
        for error in errors {
            let _ = writeln!(error_buf, "{}", error);
        }
        Err(io::Error::new(ErrorKind::Other, "Some errors occurred."))
    } else {
        if Path::new("errors.txt").is_file() {
            fs::remove_file("errors.txt")?;
        }
        Ok(())
    }
}

static CONFIG: &str = r#"title = "Comic Website"

[[comics]]
folder = "comic"
title = "First Comic"
thumbnail = "thumbnails/example.png"
description = """
This is just an example description.
You can write them on multiple lines like this if you use 3 quotes like this.
"""
"#;

static README: &str = r#"How to use this tool.

Make a folder for each comic inside this "input" folder, with the pages
inside. Put your pages inside the folder and name your pages in
alphabetical order. An easy way to do this is just number them all,
for example page-01.png or 01-pagetitle.png.

Edit the config.toml file. You should be able to use any text editor on
your computer. You want to make one entry for each comic. They will be
listed on the site in the order you put them here. Here's an example
that has multiple comics listed:

    title = "A Comics Site"
    copyright = "Copyright &copy; 2019 Cassie Jones"

    [[comics]]
    folder = "comic"
    thumbnail = "comic/page-01.png"
    title = "First Comic"
    description = """
    This is the description for the first comic.
    You can write it with multiple lines if you have 3 quotes like that.
    """

    [[comics]]
    folder = "comic2"
    thumbnail = "thumbnails/comic2.png"
    title = "Second Comic"
    description = """
    This example isn't in the default.
    """

Once this is set up, every time your run the program, it will build
your comic into the output folder.
"#;
