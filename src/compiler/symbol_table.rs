use std::{collections::HashMap, rc::Rc};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, EnumString, Display, PartialEq, Copy)]
pub enum Scope {
    #[strum(serialize = "global")]
    Global,
    #[strum(serialize = "local")]
    Local,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope: Scope,
    pub index: usize,
}

pub struct SymbolTable {
    pub symbols: HashMap<String, Rc<Symbol>>,
    pub num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            symbols: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: String) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol {
            name: name.clone(),
            scope: Scope::Global,
            index: self.num_definitions,
        });
        self.symbols.insert(name.clone(), symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Rc<Symbol>> {
        let object = self.symbols.get(name);
        match object {
            Some(symbol) => Some(symbol.clone()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_defines_symbols() {
        let mut symbol_table = SymbolTable::new();

        let expected = vec![
            Symbol {
                name: "a".to_string(),
                scope: Scope::Global,
                index: 0,
            },
            Symbol {
                name: "b".to_string(),
                scope: Scope::Global,
                index: 1,
            },
        ];

        let a = symbol_table.define("a".to_string());
        let b = symbol_table.define("b".to_string());

        assert_eq!(symbol_table.num_definitions, 2);
        assert_eq!(*a, expected[0]);
        assert_eq!(*b, expected[1]);

        for symbol in expected {
            let result = symbol_table.resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }
    }
}
