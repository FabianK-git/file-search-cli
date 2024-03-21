use std::{
    fs,
    env,
    result::Result,
    collections::HashMap, 
    process::exit,
    path::{PathBuf, Path},
    io::stdout,
};
use regex::Regex;
use crossterm::{
    cursor::MoveRight, 
    execute, 
    style::{Color, Print, ResetColor, SetForegroundColor}, 
    terminal::{
        window_size,
        Clear,
        ClearType,
        DisableLineWrap,
        EnableLineWrap,
        WindowSize
    }
};
use terminal_link::Link;
use ctrlc;
use normpath::PathExt;
use mime_guess;

struct SearchTerm {
    regex: Option<Regex>,
    search_string: Option<String>,
    mime: String
}

fn main() -> Result<(), std::io::Error> {
    // Setup Ctrl + C interrupt handling
    ctrlc::set_handler(move || {
        execute!(
            stdout(),
            SetForegroundColor(Color::Red),
            Print("\r\nKeyboard Interrupt\n"),
            ResetColor,
        ).unwrap();

        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    // Parse arguments
    let args = parse_arguments(env::args());

    // If not search option is provided print message to user
    if !(args.contains_key("name") || args.contains_key("regex") || args.contains_key("mime")) {
        execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Yellow),
            Print("No name, pattern or mime was provided.\nUse --help to get usage.\n"),
            ResetColor
        )?;

        exit(-1);
    }

    // Start search if pattern, name or mime type is specified
    // Set default values
    let mut directory = env::current_dir()?;
    let mut search: SearchTerm = SearchTerm {
        regex: None,
        search_string: None,
        mime: String::from("")
    };

    if args.contains_key("path") {
        directory = Path::new(args.get("path").unwrap()).to_path_buf();
    }

    if args.contains_key("name") {
        search.search_string = Some(String::from(args.get("name").unwrap()));
    }

    if args.contains_key("regex") {
        search.regex = Some(Regex::new(args.get("regex").unwrap()).unwrap());
    }

    if args.contains_key("mime") {
        search.mime = String::from(args.get("mime").unwrap());
    }

    let mut count = 0;

    traverse_filesystem(directory, &search, &mut count);

    execute!(
        stdout(),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::DarkGreen),
        Print("\rProcess finished\n"),
    )?;

    return Ok(());
}

fn parse_arguments(args: env::Args) -> HashMap<String, String> {
    let mut args: Vec<String> = args.collect();

    let mut arguments: HashMap<String, String> = HashMap::new();

    if args.len() <= 2 || args[1] == "--help" || args[1] == "-h" {
        execute!(
            stdout(),
            Print("Usage of find-rs:\n\n"),

            SetForegroundColor(Color::DarkGreen),
            Print(" --help, -h\t\t\t"),
            ResetColor,
            Print("Shows this output.\n"),

            SetForegroundColor(Color::DarkGreen),
            Print(" --path, -p [PATH]\t\t"),
            ResetColor,
            Print("Specifies the directory where the programs starts. (Optional)\n"),

            Print("\nSearch options:\n"),
            SetForegroundColor(Color::DarkGreen),
            Print(" --name, -n [FILENAME]\t\t"),
            ResetColor,
            Print("Specifies the name or a part of the name which should be searched.\n"),

            SetForegroundColor(Color::DarkGreen),
            Print(" --regex, -r [PATTERN]\t\t"),
            ResetColor,
            Print("Specifies a regex pattern that will be used for the search.\n"),

            SetForegroundColor(Color::DarkGreen),
            Print(" --mime, -m [MIME TYPE]\t\t"),
            ResetColor,
            Print("Specify mime type like image/png.\n"),
            Print("\t\t\t\tCan be entire mime type or part of it like \"--mime image\".\n"),
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
                "m" => "mime",
                _ => ""
            });

            arguments.insert(key, args[i + 1].clone());
        }
    }

    return arguments;
}

fn traverse_filesystem(current_dir: PathBuf, search: &SearchTerm, count: &mut usize) {
    let entries = match fs::read_dir(current_dir) {
        Ok(entries) => entries,
        Err(_) => return
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                execute!(
                    stdout(),
                    Clear(ClearType::CurrentLine),
                    SetForegroundColor(Color::Red),
                    Print(format!("\r{:?}\n", e)),
                    ResetColor
                ).unwrap();

                continue;
            }
        };

        let search_regex: bool = search.regex
            .as_ref()
            .is_some_and(|pattern| 
                 pattern.is_match(entry.file_name().to_str().unwrap())
            );

        let search_term: bool = search.search_string
            .as_ref()
            .is_some_and(|string| {
                match entry.file_name().to_str().unwrap().to_string().find(string) {
                    Some(_) => true,
                    None => false
                }
            });

        let file_path = entry.path();
        let mime_type = mime_guess::from_path(&file_path)
            .first_raw()
            .unwrap_or("");

        let search_mime: bool = match mime_type.find(&search.mime.clone()) {
            Some(_) => true,
            None => false
        };

        // If only the mime type is specified search with it
        let mime_search = search.search_string.is_none() && 
            search.search_string.is_none();

        if (search_regex || search_term || mime_search) && search_mime {
            let absolute_path = file_path.normalize();

            match absolute_path {
                Ok(absolute_path) => {
                    let path = absolute_path.as_path().to_str().unwrap_or("");

                    let file_path = format!("file://{}", path);
                    let path_link = Link::new(path, &file_path);

                    execute!(
                        stdout(),
                        Clear(ClearType::CurrentLine),
                        SetForegroundColor(Color::DarkMagenta),
                        Print(format!("\r{}", mime_type)),
                        ResetColor,
                        SetForegroundColor(Color::Green),
                        Print(format!(" {}\n", path_link)),
                        ResetColor
                    ).unwrap();
                },
                Err(e) => {
                    let path = file_path.to_str().unwrap_or("");

                    execute!(
                        stdout(),
                        Clear(ClearType::CurrentLine),
                        SetForegroundColor(Color::Red),
                        Print(format!("\r{}: {}\n", path, e)),
                        ResetColor
                    ).unwrap();
                }
            }
        }

        let size = window_size().unwrap_or(
            WindowSize {
                rows: 0,
                columns: 0,
                width: 0,
                height: 0
            }
        );

        *count += 1;
        let str_count = format!("{}", count);

        let mut move_amount = 0;

        if size.columns > str_count.len() as u16 {
            move_amount = size.columns - str_count.len() as u16;
        }

        execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            DisableLineWrap,
            Print("\r"),
            MoveRight(move_amount),
            Print(str_count),
            Print("\r"),
            Print(
                format!(
                    "\rChecking item: {}", 
                    entry.file_name().to_str().unwrap()
                )
            ),
            EnableLineWrap
        ).unwrap();

        // Call this function again if a folder is found
        if let Ok(entry_type) = entry.file_type() {
            if entry_type.is_dir() {
                traverse_filesystem(entry.path(), search, count);
            }
            else if entry_type.is_symlink() {
                if let Ok(absolute_path) = fs::canonicalize(entry.path()) {
                    let metadata = fs::metadata(absolute_path.clone());

                    if let Ok(entry_type) = metadata {
                        if entry_type.is_dir() {
                            traverse_filesystem(entry.path(), search, count);
                        }
                    }
                }
            }
        }
    }
}

