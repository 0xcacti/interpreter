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

pub struct SymbolTable<'a> {
    pub outer: Option<&'a SymbolTable<'a>>,
    pub symbols: HashMap<String, Rc<Symbol>>,
    pub num_definitions: usize,
}

impl<'a> SymbolTable<'a> {
    pub fn new() -> SymbolTable<'a> {
        SymbolTable {
            outer: None,
            symbols: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn new_enclosed(parent: &'a SymbolTable) -> SymbolTable<'a> {
        SymbolTable {
            outer: Some(parent),
            symbols: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: String) -> Rc<Symbol> {
        let scope = match self.outer {
            Some(_) => Scope::Local,
            None => Scope::Global,
        };

        let symbol = Rc::new(Symbol {
            name: name.clone(),
            scope,
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
            None => match self.outer {
                Some(outer) => outer.resolve(name),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_defines_symbols() {
        let mut global = SymbolTable::new();
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

        let expected_two = vec![
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
            Symbol {
                name: "c".to_string(),
                scope: Scope::Local,
                index: 0,
            },
            Symbol {
                name: "d".to_string(),
                scope: Scope::Local,
                index: 1,
            },
        ];
        let expected_three = vec![
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
            Symbol {
                name: "e".to_string(),
                scope: Scope::Local,
                index: 0,
            },
            Symbol {
                name: "f".to_string(),
                scope: Scope::Local,
                index: 1,
            },
        ];

        let a = global.define("a".to_string());
        let b = global.define("b".to_string());

        assert_eq!(global.num_definitions, 2);
        assert_eq!(*a, expected[0]);
        assert_eq!(*b, expected[1]);

        for symbol in expected.clone() {
            let result = global.resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }

        let mut local = SymbolTable::new_enclosed(&global);

        let c = local.define("c".to_string());
        let d = local.define("d".to_string());

        assert_eq!(local.num_definitions, 2);
        assert_eq!(*c, expected_two[2]);
        assert_eq!(*d, expected_two[3]);

        for symbol in expected_two {
            let result = local.resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }

        let mut local_local = SymbolTable::new_enclosed(&local);

        let e = local_local.define("e".to_string());
        let f = local_local.define("f".to_string());

        assert_eq!(local_local.num_definitions, 2);
        assert_eq!(*e, expected_three[2]);
        assert_eq!(*f, expected_three[3]);

        for symbol in expected_three {
            let result = local_local.resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }
    }

    #[test]
    fn it_resolves_locals() {
        let mut global_table = SymbolTable::new();
        global_table.define("a".to_string());
        global_table.define("b".to_string());

        let mut local_table = SymbolTable::new_enclosed(&global_table);
        local_table.define("c".to_string());
        local_table.define("d".to_string());

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
            Symbol {
                name: "c".to_string(),
                scope: Scope::Local,
                index: 0,
            },
            Symbol {
                name: "d".to_string(),
                scope: Scope::Local,
                index: 1,
            },
        ];

        for symbol in expected {
            let result = local_table.resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }
    }

    #[test]
    fn it_resolves_nested_loc() {
        let mut global_table = SymbolTable::new();
        global_table.define("a".to_string());
        global_table.define("b".to_string());

        let mut local_table = SymbolTable::new_enclosed(&global_table);
        local_table.define("c".to_string());
        local_table.define("d".to_string());

        let mut local_local_table = SymbolTable::new_enclosed(&local_table);
        local_local_table.define("e".to_string());
        local_local_table.define("f".to_string());

        let expected_first = vec![
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
            Symbol {
                name: "c".to_string(),
                scope: Scope::Local,
                index: 0,
            },
            Symbol {
                name: "d".to_string(),
                scope: Scope::Local,
                index: 1,
            },
        ];

        for symbol in expected_first {
            let result = local_table.resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }

        let expected_second = vec![
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
            Symbol {
                name: "e".to_string(),
                scope: Scope::Local,
                index: 0,
            },
            Symbol {
                name: "f".to_string(),
                scope: Scope::Local,
                index: 1,
            },
        ];

        for symbol in expected_second {
            let result = local_local_table.resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }
    }
}
