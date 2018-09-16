use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct SymbolTable {
    symbols: HashMap<String, Address<'static>>,
    pub current_address: u16,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Address<'a> {
    Relative(&'a str),
    Absolute(u16),
}

impl SymbolTable {
    const STARTINGTABLE: &'static [(&'static str, &'static Address<'static>)] = &[
        ("local", &Address::Relative("LCL")),
        ("argument", &Address::Relative("ARG")),
        ("this", &Address::Relative("THIS")),
        ("that", &Address::Relative("THAT")),
        ("temp", &Address::Absolute(5)),
        ("static", &Address::Absolute(16)),
    ];

    pub fn new() -> SymbolTable {
        SymbolTable {
            symbols: HashMap::new(),
            current_address: 16,
        }
    }

    pub fn load_starting_table(&mut self) {
        for entry in SymbolTable::STARTINGTABLE {
            self.add_entry(entry.0, *entry.1);
        }
    }

    pub fn add_entry(&mut self, symbol: &str, address: Address<'static>) {
        self.symbols.insert(symbol.to_string(), address);
    }

    pub fn contains(&self, symbol: &str) -> bool {
        self.symbols.contains_key(symbol)
    }

    pub fn get_address(&self, symbol: &str) -> Option<&Address<'static>> {
        self.symbols.get(symbol)
    }

    pub fn get_free_address(&self) -> u16 {
        self.current_address
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn symboltable_new() {
        let st = SymbolTable::new();
        assert_eq!(st,SymbolTable{symbols: HashMap::new(), current_address: 16});
    }

    #[test]
    fn symboltable_load_starting_table() {
        let mut st: SymbolTable = SymbolTable::new();
        st.load_starting_table();
        assert_eq!(st.get_address("static").unwrap(), &Address::Absolute(16));
    }

    #[test]
    fn symboltable_add_entry() {
        let mut st: SymbolTable = SymbolTable::new();
        st.add_entry("TestAddress", Address::Absolute(12345));
        assert_eq!(st.get_address("TestAddress").unwrap(), &Address::Absolute(12345));
    }

    #[test]
    fn symboltable_contains() {
        let mut st: SymbolTable = SymbolTable::new();
        st.add_entry("TestAddress", Address::Absolute(12345));
        assert_eq!(st.contains("TestAddress"), true);
    }
}
