use std::collections::HashMap;
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
    pub value: usize,
}

pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>,
    pub num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            symbols: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: String, scope: Scope, value: usize) -> Symbol {
        let symbol = Symbol {
            name: name.clone(),
            scope,
            value,
        };
        self.symbols
            .insert(name.clone(), Symbol { name, scope, value });
        self.num_definitions += 1;
        symbol.clone()
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
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
                value: 0,
            },
            Symbol {
                name: "b".to_string(),
                scope: Scope::Global,
                value: 1,
            },
        ];

        let a = symbol_table.define("a".to_string(), Scope::Global, 0);
        let b = symbol_table.define("b".to_string(), Scope::Global, 1);

        assert_eq!(symbol_table.num_definitions, 2);
        assert_eq!(a, expected[0],);
        assert_eq!(b, expected[1],);

        for symbol in expected {
            let result = symbol_table.resolve(&symbol.name).unwrap();
            assert_eq!(result, symbol);
        }
    }
}
