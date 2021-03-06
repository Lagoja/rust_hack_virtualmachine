use lib::parser::{Command, Parser};
use lib::symbol_table::SymbolTable;
use lib::tokenizer::{default_ruleset, TokenList, Tokenizer};
use lib::writer::AsmWriter;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::io::{BufReader, Result as IOResult};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub filevec: Vec<PathBuf>,
    pub outfile: PathBuf,
    pub write_init: bool,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, Box<Error>> {
        args.next();

        let path = match args.next() {
            Some(arg) => PathBuf::from(arg),
            None => {
                return Err(Box::new(FileTypeError));
            }
        };

        let write_init = match args.next() {
            Some(arg) => match arg.as_ref() {
                "--no-init" => false,
                _ => return Err(Box::new(InvalidArgError)),
            },
            None => true,
        };

        let of = path.clone();
        let outfile = PathBuf::from(of.with_extension("asm"));

        let filevec: Vec<PathBuf> = match path.is_dir() {
            true => get_vmfiles_in_path(path)?,
            false => match &path.extension() {
                Some(x) => {
                    if x.to_str().unwrap() == "vm" {
                        println!("Adding File: {}", path.to_str().unwrap());
                        vec![path.clone()]
                    } else {
                        return Err(Box::new(FileTypeError));
                    }
                }
                None => return Err(Box::new(FileTypeError)),
            },
        };

        Ok(Config {
            filevec,
            outfile,
            write_init,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let mut file_map: HashMap<String, Vec<String>> = HashMap::new();

    for filename in config.filevec {
        println!("Loading file {}", filename.to_str().unwrap());
        let f: fs::File = fs::File::open(&filename)?;
        let br = BufReader::new(f);
        let raw_commands: Vec<String> = br
            .lines()
            .map(|l| l.expect("Could not load file"))
            .collect();
        file_map.insert(
            String::from(filename.file_stem().unwrap().to_string_lossy()),
            raw_commands,
        );
    }

    let mut st: SymbolTable = SymbolTable::new();
    st.load_starting_table();
    let mut writer: AsmWriter = AsmWriter::from(st);

    let mut tokens: HashMap<String, Vec<TokenList>> = HashMap::new();

    for (filename, raw_commands) in file_map {
        let tokenizer = Tokenizer::from(default_ruleset());
        tokens.insert(
            filename,
            raw_commands
                .into_iter()
                .map(|string| tokenizer.tokenize(&string).unwrap())
                .collect(),
        );
    }

    let mut cl: Vec<Command> = vec![];
    for (filename, line) in tokens {
        let mut parser = Parser::from(line, filename);
        while parser.has_more_commands() {
            match parser.advance()? {
                Some(comm) => cl.push(comm),
                None => continue,
            };
        }
    }

    let mut out: Vec<String> = vec![];

    if config.write_init {
        out.push(writer.write_init().unwrap());
    }

    out.push(
        cl.into_iter()
            .map(|comm| writer.write_command(comm).unwrap())
            .collect(),
    );

    write_asm_file(out.join(""), &config.outfile).unwrap();

    Ok(())
}

fn write_asm_file(machine_code: String, path_name: &PathBuf) -> Result<(), Box<Error>> {
    let mut f = fs::File::create(path_name)?;
    f.write_all(machine_code.as_bytes())?;
    Ok(())
}

fn get_vmfiles_in_path(path: PathBuf) -> IOResult<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = vec![];
    let dir_res = fs::read_dir(&path)?
        .map(|result| result.map(|entry| entry.path()))
        .collect::<Result<Vec<PathBuf>, _>>()?;

    for path in dir_res {
        if let Some(ext) = &path.extension() {
            if let Some(ext_str) = ext.to_str() {
                println!("Extension: {}", ext_str);
                if ext_str == "vm" {
                    out.push(path.clone());
                }
            }
        }
    }
    Ok(out)
}

#[derive(Debug)]
struct FileTypeError;

impl fmt::Display for FileTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Please provide a .vm file or directory")
    }
}

impl Error for FileTypeError {}

#[derive(Debug)]
struct InvalidArgError;

impl fmt::Display for InvalidArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid command or option")
    }
}

impl Error for InvalidArgError {}
