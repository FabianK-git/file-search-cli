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
use normpath::PathExt;

struct SearchTerm {
    regex: Option<Regex>,
    search_string: Option<String>
}

fn main() -> Result<()> {
    // Setup Ctrl + C interrupt handling
    ctrlc::set_handler(move || {
        execute!(
            stdout(),
            SetForegroundColor(Color::Red),
            Print("\r\nKeyboard Interrupt\n"),
            ResetColor
        ).unwrap();

        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    // Parse arguments
    let args = parse_arguments(env::args());

    // Start search if pattern or name is specified
    if args.contains_key("name") || args.contains_key("regex") {
        // Set default values
        let mut directory = env::current_dir()?;
        let mut search: SearchTerm = SearchTerm { regex: None, search_string: None };

        if args.contains_key("path") {
            directory = Path::new(args.get("path").unwrap()).to_path_buf();
        }

        if args.contains_key("name") {
            search.search_string = Some(String::from(args.get("name").unwrap()));
        }

        if args.contains_key("regex") {
            search.regex = Some(Regex::new(args.get("regex").unwrap()).unwrap());
        }

        traverse_filesystem(directory, &search);

        execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::DarkGreen),
            Print("\rProcess finished\n")
        )?;
    }
    else {
        execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Yellow),
            Print("No name or pattern was provided.\nUse --help to get usage.\n"),
            ResetColor
        )?;
    }

    return Ok(());
}

fn parse_arguments(args: env::Args) -> HashMap<String, String> {
    let mut args: Vec<String> = args.collect();

    let mut arguments: HashMap<String, String> = HashMap::new();

    if args.len() <= 2 || args[1] == "--help" || args[1] == "-h" {
        execute!(
            stdout(),
            Print("Usage of find-rs:\n"),

            SetForegroundColor(Color::DarkGreen),
            Print("--help, -h\t\t\t"),
            ResetColor,
            Print("Shows this output.\n"),

            SetForegroundColor(Color::DarkGreen),
            Print("--path, -p [PATH]\t\t"),
            ResetColor,
            Print("Specifies the directory where the programs starts. (Optional)\n"),

            SetForegroundColor(Color::DarkGreen),
            Print("--name, -n [FILENAME]\t\t"),
            ResetColor,
            Print("Specifies the name or a part of the name which should be searched.\n"),

            SetForegroundColor(Color::DarkGreen),
            Print("--regex, -r [PATTERN]\t\t"),
            ResetColor,
            Print("Specifies a regex pattern that will be used for the search.\n"),
        ).unwrap();

        // Exit this program early
        exit(0);
    }

    for i in 0..args.len() {
        if (args[i].starts_with("--") ) && i + 1 < args.len() {
            args[i].replace_range(0..2, "");
            
            let key: String = args[i].clone();

            arguments.insert(key, args[i + 1].clone());
        }
        else if args[i].starts_with("-") && i + 1 < args.len() {
            args[i].replace_range(0..1, "");
            
            let key: String = String::from(match args[i].clone().as_str() {
                "h" => "help",
                "p" => "path",
                "n" => "name",
                "r" => "regex",
                _ => ""
            });

            arguments.insert(key, args[i + 1].clone());
        }
    }

    return arguments;
}

fn traverse_filesystem(current_dir: PathBuf, search: &SearchTerm) {
    let entries = fs::read_dir(current_dir);
    
    if let Ok(entries) = entries {
        for entry in entries {
            match entry {
                Ok(entry) => {
                    if search.regex.as_ref().is_some_and(|pattern| pattern.is_match(entry.file_name().to_str().unwrap())) || 
                       search.search_string.as_ref().is_some_and(|string| string.find(entry.file_name().to_str().unwrap()).is_some()) 
                    {
                        // if search.search_string.is_some() {
                        //     println!("{:?}", search.search_string);
                        // }

                        let file_path = entry.path();
                        let absolute_path = file_path.normalize();

                        match absolute_path {
                            Ok(absolute_path) => {
                                let path = absolute_path.as_path().to_str().unwrap_or("");

                                execute!(
                                    stdout(),
                                    Clear(ClearType::CurrentLine),
                                    SetForegroundColor(Color::Green),
                                    Print(format!("\r{}\n", Link::new(path, &format!("file://{}", path)))),
                                    ResetColor
                                ).unwrap();
                            },
                            Err(e) => println!("{:?}", e)
                        }
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
                            traverse_filesystem(entry.path(), search);
                        }
                    }
                },
                Err(e) => println!("\n{:?}", e)
            };
        }
    }
}
