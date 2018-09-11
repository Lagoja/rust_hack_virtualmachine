use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct SymbolTable {
    symbols: HashMap<String, u16>,
    pub current_address: u16,
}

impl SymbolTable {
    const STARTINGTABLE: &'static [(&'static str, &'static u16)] = &[
        ("SP", &0),
        ("LCL", &1),
        ("ARG", &2),
        ("THIS", &3),
        ("THAT", &4),
        ("R0", &0),
        ("R1", &1),
        ("R2", &2),
        ("R3", &3),
        ("R4", &4),
        ("R5", &5),
        ("R6", &6),
        ("R7", &7),
        ("R8", &8),
        ("R9", &9),
        ("R10", &10),
        ("R11", &11),
        ("R12", &12),
        ("R13", &13),
        ("R14", &14),
        ("R15", &15),
        ("SCREEN", &16384),
        ("KBD", &24576),
    ];

    pub fn new() -> SymbolTable {
        SymbolTable {
            symbols: HashMap::new(),
            current_address: 16,
        }
    }

    pub fn load_starting_table(&mut self) {
        for entry in SymbolTable::STARTINGTABLE {
            self.add_entry(entry.0, *entry.1)
        }
    }

    pub fn add_entry(&mut self, symbol: &str, address: u16) {
        self.symbols.insert(symbol.to_string(), address);
    }

    pub fn contains(&self, symbol: &str) -> bool {
        self.symbols.contains_key(symbol)
    }

    pub fn get_address(&self, symbol: &str) -> Option<&u16> {
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
        assert_eq!(st.get_address("SCREEN").unwrap(), &16384);
        assert_eq!(st.get_address("R0").unwrap(), &0);
        assert_eq!(st.get_address("SP").unwrap(), &0);
    }

    #[test]
    fn symboltable_add_entry() {
        let mut st: SymbolTable = SymbolTable::new();
        st.add_entry("TestAddress", 12345);
        assert_eq!(st.get_address("TestAddress").unwrap(), &12345);
    }

    #[test]
    fn symboltable_contains() {
        let mut st: SymbolTable = SymbolTable::new();
        st.add_entry("TestAddress", 12345);
        assert_eq!(st.contains("TestAddress"), true);
    }
}
