use std::collections::HashMap;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct SymbolId(usize);

#[derive(Debug, Default)]
pub struct SymbolTable<'s> {
    name_: HashMap<&'s str, SymbolId>,
    sym_: HashMap<SymbolId, Symbol<'s>>,
    next_id: usize,
    string_interner: HashMap<String, &'s str>,
}

impl<'s> SymbolTable<'s> {
    pub fn intern_str(&mut self, str: &str) -> &'s str {
        if let Some(&existing) = self.string_interner.get(str) {
            return existing;
        }

        // safe leak
        let leaked = Box::leak(str.to_string().into_boxed_str());
        self.string_interner.insert(leaked.to_string(), leaked);
        leaked
    }

    pub fn intern_string(&mut self, str: String) -> &'s str {
        if let Some(&existing) = self.string_interner.get(&str) {
            return existing;
        }

        let leaked = Box::leak(str.into_boxed_str());
        self.string_interner.insert(leaked.to_string(), leaked);
        leaked
    }

    pub fn insert(
        &mut self,
        name: &str,
        r#type: SymbolKind,
        value: Option<u32>,
        line: usize,
    ) -> SymbolId {
        if let Some(&existing) = self.name_.get(&name) {
            return existing;
        }

        let id = self.next_id();

        let interned_name = self.intern_str(name);
        let symbol = Symbol {
            name: interned_name,
            r#type,
            value,
            line,
        };

        self.name_.insert(interned_name, id);
        self.sym_.insert(id, symbol);

        id
    }

    pub fn update<F>(&mut self, id: SymbolId, predicate: F)
    where
        F: FnOnce(&mut Symbol),
    {
        if let Some(s) = self.sym_.get_mut(&id) {
            predicate(s);
        }
    }

    pub fn next_id(&mut self) -> SymbolId {
        self.next_id += 1;
        SymbolId(self.next_id - 1)
    }

    pub fn get_id(&self, n: &str) -> Option<SymbolId> {
        self.name_.get(n).copied()
    }

    pub fn get_symbol(&self, n: &SymbolId) -> Option<Symbol> {
        self.sym_.get(n).copied()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Symbol<'s> {
    pub name: &'s str,
    pub r#type: SymbolKind,
    pub value: Option<u32>,
    pub line: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum SymbolKind {
    Label,
    Directive,
    Parameter,
    #[default]
    None,
}
