use std::{cell::RefCell, collections::HashMap, rc::Rc};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, EnumString, Display, PartialEq, Copy, Eq)]
pub enum Scope {
    #[strum(serialize = "global")]
    Global,
    #[strum(serialize = "local")]
    Local,
    #[strum(serialize = "builtin")]
    Builtin,
    #[strum(serialize = "free")]
    Free,
    #[strum(serialize = "function")]
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub scope: Scope,
    pub index: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SymbolTable {
    pub outer: Option<Rc<RefCell<SymbolTable>>>,
    pub symbols: HashMap<String, Rc<Symbol>>,
    pub free_symbols: Vec<Rc<Symbol>>,
    pub num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> Rc<RefCell<SymbolTable>> {
        Rc::new(RefCell::new(SymbolTable {
            outer: None,
            symbols: HashMap::new(),
            free_symbols: vec![],
            num_definitions: 0,
        }))
    }

    pub fn new_enclosed(parent: Rc<RefCell<SymbolTable>>) -> Rc<RefCell<SymbolTable>> {
        Rc::new(RefCell::new(SymbolTable {
            outer: Some(parent),
            symbols: HashMap::new(),
            free_symbols: vec![],
            num_definitions: 0,
        }))
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

    pub fn define_free(&mut self, original: Rc<Symbol>) -> Rc<Symbol> {
        self.free_symbols.push(original.clone());
        let symbol = Rc::new(Symbol {
            name: original.name.clone(),
            scope: Scope::Free,
            index: self.free_symbols.len() - 1,
        });
        self.symbols.insert(original.name.clone(), symbol.clone());

        symbol
    }

    pub fn define_builtin(&mut self, index: usize, name: String) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol {
            name: name.clone(),
            scope: Scope::Builtin,
            index,
        });
        self.symbols.insert(name.clone(), symbol.clone());
        symbol
    }

    pub fn define_function_name(&mut self, name: String) -> Rc<Symbol> {
        let symbol = Rc::new(Symbol {
            name: name.clone(),
            scope: Scope::Function,
            index: 0,
        });
        self.symbols.insert(name, symbol.clone());

        symbol
    }

    pub fn resolve(&mut self, name: &str) -> Option<Rc<Symbol>> {
        let object = self.symbols.get(name);

        match object {
            Some(symbol) => Some(symbol.clone()),
            None => match &self.outer {
                Some(outer) => {
                    let obj = outer.borrow_mut().resolve(name);
                    match obj {
                        Some(symbol) => {
                            if symbol.scope == Scope::Global || symbol.scope == Scope::Builtin {
                                return Some(symbol);
                            }

                            let free = self.define_free(symbol);
                            return Some(free);
                        }
                        None => return None,
                    }
                }
                None => return None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;

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

        let a = global.borrow_mut().define("a".to_string());
        let b = global.borrow_mut().define("b".to_string());

        assert_eq!(global.borrow().num_definitions, 2);
        assert_eq!(*a, expected[0]);
        assert_eq!(*b, expected[1]);

        for symbol in expected.clone() {
            let result = global.borrow_mut().resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }

        let local = SymbolTable::new_enclosed(global.clone());

        let c = local.borrow_mut().define("c".to_string());
        let d = local.borrow_mut().define("d".to_string());

        assert_eq!(local.borrow().num_definitions, 2);
        assert_eq!(*c, expected_two[2]);
        assert_eq!(*d, expected_two[3]);

        for symbol in expected_two {
            let result = local.borrow_mut().resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }

        let local_local = SymbolTable::new_enclosed(local.clone());

        let e = local_local.borrow_mut().define("e".to_string());
        let f = local_local.borrow_mut().define("f".to_string());

        assert_eq!(local_local.borrow().num_definitions, 2);
        assert_eq!(*e, expected_three[2]);
        assert_eq!(*f, expected_three[3]);

        for symbol in expected_three {
            let result = local_local.borrow_mut().resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }
    }

    #[test]
    fn it_resolves_locals() {
        let global_table = SymbolTable::new();
        global_table.borrow_mut().define("a".to_string());
        global_table.borrow_mut().define("b".to_string());

        let local_table = SymbolTable::new_enclosed(global_table.clone());
        local_table.borrow_mut().define("c".to_string());
        local_table.borrow_mut().define("d".to_string());

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
            let result = local_table.borrow_mut().resolve(&symbol.name).unwrap();
            assert_eq!(*result, symbol);
        }
    }

    #[test]
    fn it_resolves_nested_locals() {
        let global_table = SymbolTable::new();
        global_table.borrow_mut().define("a".to_string());
        global_table.borrow_mut().define("b".to_string());

        let local_table = SymbolTable::new_enclosed(global_table.clone());
        local_table.borrow_mut().define("c".to_string());
        local_table.borrow_mut().define("d".to_string());

        let local_local_table = SymbolTable::new_enclosed(local_table.clone());
        local_local_table.borrow_mut().define("e".to_string());
        local_local_table.borrow_mut().define("f".to_string());

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
            let result = local_table.borrow_mut().resolve(&symbol.name).unwrap();
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
            let result = local_local_table
                .borrow_mut()
                .resolve(&symbol.name)
                .unwrap();
            assert_eq!(*result, symbol);
        }
    }

    #[test]
    fn it_resolves_builtins() {
        let global_table = SymbolTable::new();
        global_table.borrow_mut().define_builtin(0, "a".to_string());
        global_table.borrow_mut().define_builtin(1, "c".to_string());
        global_table.borrow_mut().define_builtin(2, "e".to_string());
        global_table.borrow_mut().define_builtin(3, "f".to_string());

        let local_table = SymbolTable::new_enclosed(global_table.clone());
        let local_local_table = SymbolTable::new_enclosed(local_table.clone());

        let expected = vec![
            Symbol {
                name: "a".to_string(),
                scope: Scope::Builtin,
                index: 0,
            },
            Symbol {
                name: "c".to_string(),
                scope: Scope::Builtin,
                index: 1,
            },
            Symbol {
                name: "e".to_string(),
                scope: Scope::Builtin,
                index: 2,
            },
            Symbol {
                name: "f".to_string(),
                scope: Scope::Builtin,
                index: 3,
            },
        ];

        for (i, symbol) in expected.iter().enumerate() {
            global_table
                .borrow_mut()
                .define_builtin(i, symbol.name.clone());
        }

        for table in [global_table, local_table, local_local_table].iter() {
            for symbol in expected.clone() {
                let result = table.borrow_mut().resolve(&symbol.name).unwrap();
                assert_eq!(*result, symbol);
            }
        }
    }

    #[test]
    fn it_resolves_free_scopes() {
        let global_table = SymbolTable::new();
        global_table.borrow_mut().define("a".to_string());
        global_table.borrow_mut().define("b".to_string());

        let local_table = SymbolTable::new_enclosed(global_table.clone());
        local_table.borrow_mut().define("c".to_string());
        local_table.borrow_mut().define("d".to_string());

        let local_local_table = SymbolTable::new_enclosed(local_table.clone());
        local_local_table.borrow_mut().define("e".to_string());
        local_local_table.borrow_mut().define("f".to_string());

        struct TestCase {
            table: Rc<RefCell<SymbolTable>>,
            expected_symbols: Vec<Symbol>,
            expected_free_symbols: Vec<Symbol>,
        }

        let tests = vec![
            TestCase {
                table: local_table.clone(),
                expected_symbols: vec![
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
                ],
                expected_free_symbols: vec![],
            },
            TestCase {
                table: local_local_table.clone(),
                expected_symbols: vec![
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
                        scope: Scope::Free,
                        index: 0,
                    },
                    Symbol {
                        name: "d".to_string(),
                        scope: Scope::Free,
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
                ],
                expected_free_symbols: vec![
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
                ],
            },
        ];

        for test_case in tests {
            for symbol in test_case.expected_symbols {
                let result = test_case.table.borrow_mut().resolve(&symbol.name).unwrap();
                assert_eq!(*result, symbol);
            }
            assert_eq!(
                test_case.table.borrow().free_symbols.len(),
                test_case.expected_free_symbols.len()
            );

            for (i, symbol) in test_case.expected_free_symbols.iter().enumerate() {
                let table_borrow = test_case.table.borrow();
                let result = table_borrow.free_symbols.get(i).unwrap();
                assert_eq!(**result, *symbol);
            }
        }
    }

    #[test]
    fn it_cant_resolve_unresolvable_frees() {
        let global_table = SymbolTable::new();
        global_table.borrow_mut().define_builtin(0, "a".to_string());
        global_table.borrow_mut().define("a".to_string());

        let local_table = SymbolTable::new_enclosed(global_table.clone());
        local_table.borrow_mut().define("c".to_string());

        let local_local_table = SymbolTable::new_enclosed(local_table.clone());
        local_local_table.borrow_mut().define("e".to_string());
        local_local_table.borrow_mut().define("f".to_string());

        let expected = vec![
            Symbol {
                name: "a".to_string(),
                scope: Scope::Global,
                index: 0,
            },
            Symbol {
                name: "c".to_string(),
                scope: Scope::Free,
                index: 0,
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

        for symbol in expected {
            let result = local_local_table
                .borrow_mut()
                .resolve(&symbol.name)
                .unwrap();
            assert_eq!(*result, symbol);
        }

        let unresolvable = vec!["b", "d"];

        for name in unresolvable {
            let result = local_local_table.borrow_mut().resolve(name);
            assert_eq!(result, None);
        }
    }

    #[test]
    fn it_defines_and_resolves_function_names() {
        let global_table = SymbolTable::new();
        global_table
            .borrow_mut()
            .define_function_name("a".to_string());

        let expected = Symbol {
            name: "a".to_string(),
            scope: Scope::Function,
            index: 0,
        };

        let result = global_table.borrow_mut().resolve("a").unwrap();

        assert_eq!(*result, expected);
    }

    #[test]
    fn it_correctly_shadows_function_names() {
        let global_table = SymbolTable::new();
        global_table
            .borrow_mut()
            .define_function_name("a".to_string());

        global_table.borrow_mut().define("a".to_string());

        let expected = Symbol {
            name: "a".to_string(),
            scope: Scope::Global,
            index: 0,
        };

        let result = global_table.borrow_mut().resolve("a").unwrap();

        assert_eq!(*result, expected);
    }
}
