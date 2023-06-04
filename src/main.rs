use std::{
    fs,
    env, 
    collections::HashMap, 
    process::exit,
    path::{PathBuf, Path},
    io::stdout
};
use regex::Regex;
use crossterm::{
    execute, 
    style::{Color, Print, ResetColor, SetForegroundColor},
    Result,
    terminal::{Clear, ClearType, DisableLineWrap, EnableLineWrap}
};
use terminal_link::Link;
use ctrlc;

fn main() -> Result<()> {
    // Setup Ctrl + C interrupt handling
    ctrlc::set_handler(move || {
        execute!(
            stdout(),
            SetForegroundColor(Color::Red),
            Print("\r\nKeyboard Interrupt"),
            ResetColor
        ).unwrap();

        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    // Parse arguments
    let args = parse_arguments(env::args());

    // Start search if pattern or name is specified
    if args.contains_key("name") || args.contains_key("pattern") {
        // Set default values
        let mut directory = env::current_dir()?;
        let mut pattern: Option<Regex> = None;

        if args.contains_key("dir") {
            directory = Path::new(args.get("dir").unwrap()).to_path_buf();
        }

        if args.contains_key("name") {
            pattern = Some(Regex::new(args.get("name").unwrap()).unwrap());
        }

        if args.contains_key("pattern") {
            pattern = Some(Regex::new(args.get("pattern").unwrap()).unwrap());
        }

        if let Some(pattern) = pattern {
            traverse_filesystem(directory, &pattern);
        }

        execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::DarkGreen),
            Print("\rProcess finished")
        )?;
    }
    else {
        execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Yellow),
            Print("No name or pattern was provided.\nUse --help to get usage."),
            ResetColor
        )?;
    }

    return Ok(());
}

fn parse_arguments(args: env::Args) -> HashMap<String, String> {
    let mut args: Vec<String> = args.collect();

    let mut arguments: HashMap<String, String> = HashMap::new();

    if args.len() <= 2 || args[1] == "--help" {
        execute!(
            stdout(),
            Print("Usage of find-rs:\n"),

            SetForegroundColor(Color::Grey),
            Print("--help\t\t\t"),
            ResetColor,
            Print("Shows this output.\n"),

            SetForegroundColor(Color::Grey),
            Print("--dir PATH\t\t"),
            ResetColor,
            Print("Specifies the directory where the programs starts. (Optional)\n"),

            SetForegroundColor(Color::Grey),
            Print("--name FILENAME\t\t"),
            ResetColor,
            Print("Specifies the name or a part of the name which should be searched.\n"),

            SetForegroundColor(Color::Grey),
            Print("--pattern REGEX-PATTERN\t"),
            ResetColor,
            Print("Specifies a regex pattern that will be used for the search.\n"),
        ).unwrap();

        // Exit this program early
        exit(0);
    }

    for i in 0..args.len() {
        if args[i].starts_with("--") && i + 1 < args.len() {
            args[i].replace_range(0..2, "");
            
            let key: String = args[i].clone();

            arguments.insert(key, args[i + 1].clone());
        }
    }

    return arguments;
}

fn traverse_filesystem(current_dir: PathBuf, pattern: &Regex) {
    let entries = fs::read_dir(current_dir);

    if let Ok(entries) = entries {
        for entry in entries {
            match entry {
                Ok(entry) => {
                    if pattern.is_match(entry.file_name().to_str().unwrap()) {
                        let filename = entry.path().to_str().unwrap_or("").to_owned();

                        execute!(
                            stdout(),
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(Color::Green),
                            Print(format!("\r{}\n", Link::new(&filename, &format!("file://{}", filename)))),
                            ResetColor
                        ).unwrap();
                    }

                    execute!(
                        stdout(),
                        Clear(ClearType::CurrentLine),
                        DisableLineWrap,
                        Print(format!("\rChecking item: {}", entry.file_name().to_str().unwrap())),
                        EnableLineWrap
                    ).unwrap();

                    // Call this function again if a folder is found
                    if let Ok(entry_type) = entry.file_type() {
                        if entry_type.is_dir() {
                            traverse_filesystem(entry.path(), pattern);
                        }
                    }
                },
                Err(e) => println!("\n{:?}", e)
            };
        }
    }
}
