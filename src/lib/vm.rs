use lib::writer::AsmWriter;
use lib::symbol_table::SymbolTable;
use lib::tokenizer::{default_ruleset, TokenList, Tokenizer};
use lib::parser::{Parser, Command};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub filename: PathBuf,
    pub outfile: PathBuf,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let filename = match args.next() {
            Some(arg) => PathBuf::from(arg),
            None => {
                return Err("No filename provided");
            }
        };

        match filename.extension() {
            Some(x) => {
                if x != "vm" {
                    return Err("Please provide a .vm file");
                }
            }
            None => return Err("Please provide a .vm file"),
        }

        let of = filename.clone();
        let outfile = PathBuf::from(of.with_extension("asm"));

        Ok(Config { filename, outfile })
    }
}

pub fn run(config: Config) -> Result<(), Box<Error>> {
    let f: File = File::open(&config.filename)?;
    let br = BufReader::new(f);
    let raw_commands: Vec<String> = br
        .lines()
        .map(|l| l.expect("Could not load file"))
        .collect();

    let mut st: SymbolTable = SymbolTable::new();
    st.load_starting_table();

    let tokenizer = Tokenizer::from(default_ruleset());
    let writer: AsmWriter = AsmWriter::from(st);

    let tokens: Vec<TokenList> = raw_commands
        .into_iter()
        .map(|string| tokenizer.tokenize(&string).unwrap())
        .collect();

    let mut parser = Parser::from(tokens);
    let mut cl: Vec<Command> = vec![];

    while parser.has_more_commands(){
        let comm = match parser.advance()?{
            Some(comm) => cl.push(comm),
            None => continue,
        };
    }

    let out: Vec<String> = cl.into_iter().map(|comm| writer.write_command(comm).unwrap()).collect();

    write_asm_file(out.join(""), &config.outfile).unwrap();

    Ok(())
}

fn write_asm_file(machine_code: String, path_name: &PathBuf) -> Result<(), Box<Error>> {
    let mut f = File::create(path_name)?;
    f.write_all(machine_code.as_bytes())?;
    Ok(())
}
