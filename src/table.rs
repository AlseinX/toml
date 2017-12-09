use linked_hash_map::LinkedHashMap;
use value::{sort_key_value_pairs, Array, DateTime, InlineTable, Value};
use decor::{Decor, InternalString, Repr};
use key::Key;
use array_of_tables::ArrayOfTables;
use formatted::{decorated, key_repr};

// TODO: add method to convert a table into inline table
// TODO: documentation

/// Type representing a TOML non-inline table
#[derive(Clone, Debug, Default)]
pub struct Table {
    pub(crate) items: KeyValuePairs,
    // comments/spaces before and after the header
    pub(crate) decor: Decor,
    // whether to hide an empty table
    pub(crate) implicit: bool,
}

pub(crate) type KeyValuePairs = LinkedHashMap<InternalString, TableKeyValue>;

#[derive(Debug, Clone)]
pub enum Item {
    None,
    Value(Value),
    Table(Table),
    ArrayOfTables(ArrayOfTables),
}

impl Default for Item {
    fn default() -> Self {
        Item::None
    }
}

// TODO: make pub(crate)
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct TableKeyValue {
    pub(crate) key: Repr,
    pub(crate) value: Item,
}

impl TableKeyValue {
    pub(crate) fn new(key: Repr, value: Item) -> Self {
        TableKeyValue {
            key: key,
            value: value,
        }
    }
}

pub type Iter<'a> = Box<Iterator<Item = (&'a str, &'a Item)> + 'a>;

impl Table {
    pub fn new() -> Self {
        Self::with_decor(Decor::new("\n", ""))
    }

    pub(crate) fn with_decor(decor: Decor) -> Self {
        Self {
            decor: decor,
            ..Default::default()
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        if let Some(kv) = self.items.get(key) {
            !kv.value.is_none()
        } else {
            false
        }
    }

    pub fn contains_table(&self, key: &str) -> bool {
        if let Some(kv) = self.items.get(key) {
            kv.value.is_table()
        } else {
            false
        }
    }

    pub fn contains_value(&self, key: &str) -> bool {
        if let Some(kv) = self.items.get(key) {
            kv.value.is_value()
        } else {
            false
        }
    }

    pub fn contains_array_of_tables(&self, key: &str) -> bool {
        if let Some(kv) = self.items.get(key) {
            kv.value.is_array_of_tables()
        } else {
            false
        }
    }

    pub fn iter(&self) -> Iter {
        Box::new(self.items.iter().map(|(key, kv)| (&key[..], &kv.value)))
    }

    pub fn remove(&mut self, key: &str) -> Option<Item> {
        self.items.remove(key).map(|kv| kv.value)
    }

    /// Sorts Key/Value Pairs of the table,
    /// doesn't affect subtables or subarrays.
    pub fn sort_values(&mut self) {
        sort_key_value_pairs(&mut self.items);
    }

    /// Returns the number of non-empty items in the table.
    pub fn len(&self) -> usize {
        self.items.iter().filter(|i| !(i.1).value.is_none()).count()
    }

    /// Returns the number of key/value pairs in the table.
    pub fn values_len(&self) -> usize {
        self.items.iter().filter(|i| (i.1).value.is_value()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Given the `key`, return a mutable reference to the value.
    /// If there is no entry associated with the given key in the table,
    /// a `Item::None` value will be inserted.
    ///
    /// To insert to item, use `entry` to return a mutable reference
    /// and set it to the appropriate value.
    pub fn entry<'a>(&'a mut self, key: &str) -> &'a mut Item {
        let parsed_key = key.parse::<Key>().expect("invalid key");
        &mut self.items
            .entry(parsed_key.get().to_owned())
            .or_insert(TableKeyValue::new(key_repr(parsed_key.raw()), Item::None))
            .value
    }

    pub fn get<'a>(&'a self, key: &str) -> Option<&'a Item> {
        self.items.get(key).map(|kv| &kv.value)
    }

    /// If a table has no key value pairs and implicit, it will not be displayed.
    ///
    /// # Examples
    ///
    /// ```notrust
    /// [target."x86_64/windows.json".dependencies]
    /// ```
    ///
    /// In the document above, tables `target` and `target."x86_64/windows.json"` are implicit.
    ///
    /// ```
    /// # extern crate toml_edit;
    /// # use toml_edit::Document;
    /// #
    /// # fn main() {
    /// let mut doc = "[a]\n[a.b]\n".parse::<Document>().expect("valid toml");
    ///
    /// doc.root.entry("a").as_table_mut().unwrap().set_implicit(true);
    /// assert_eq!(doc.to_string(), "[a.b]\n");
    /// # }
    /// ```
    pub fn set_implicit(&mut self, implicit: bool) {
        self.implicit = implicit;
    }
}

impl Item {
    pub fn or_insert(&mut self, item: Item) -> &mut Item {
        if self.is_none() {
            *self = item
        }
        self
    }
}
// TODO: This should be generated by macro or derive
/// Downcasting
impl Item {
    pub fn as_value(&self) -> Option<&Value> {
        match *self {
            Item::Value(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn as_table(&self) -> Option<&Table> {
        match *self {
            Item::Table(ref t) => Some(t),
            _ => None,
        }
    }
    pub fn as_array_of_tables(&self) -> Option<&ArrayOfTables> {
        match *self {
            Item::ArrayOfTables(ref a) => Some(a),
            _ => None,
        }
    }
    pub fn as_value_mut(&mut self) -> Option<&mut Value> {
        match *self {
            Item::Value(ref mut v) => Some(v),
            _ => None,
        }
    }
    pub fn as_table_mut(&mut self) -> Option<&mut Table> {
        match *self {
            Item::Table(ref mut t) => Some(t),
            _ => None,
        }
    }
    pub fn as_array_of_tables_mut(&mut self) -> Option<&mut ArrayOfTables> {
        match *self {
            Item::ArrayOfTables(ref mut a) => Some(a),
            _ => None,
        }
    }
    pub fn is_value(&self) -> bool {
        self.as_value().is_some()
    }
    pub fn is_table(&self) -> bool {
        self.as_table().is_some()
    }
    pub fn is_array_of_tables(&self) -> bool {
        self.as_array_of_tables().is_some()
    }
    pub fn is_none(&self) -> bool {
        match *self {
            Item::None => true,
            _ => false,
        }
    }

    // Duplicate Value downcasting API

    pub fn as_integer(&self) -> Option<i64> {
        self.as_value().and_then(|v| v.as_integer())
    }

    pub fn is_integer(&self) -> bool {
        self.as_integer().is_some()
    }

    pub fn as_float(&self) -> Option<f64> {
        self.as_value().and_then(|v| v.as_float())
    }

    pub fn is_float(&self) -> bool {
        self.as_float().is_some()
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.as_value().and_then(|v| v.as_bool())
    }

    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn as_str(&self) -> Option<&str> {
        self.as_value().and_then(|v| v.as_str())
    }

    pub fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn as_date_time(&self) -> Option<&DateTime> {
        self.as_value().and_then(|v| v.as_date_time())
    }

    pub fn is_date_time(&self) -> bool {
        self.as_date_time().is_some()
    }

    pub fn as_array(&self) -> Option<&Array> {
        self.as_value().and_then(|v| v.as_array())
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Array> {
        self.as_value_mut().and_then(|v| v.as_array_mut())
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn as_inline_table(&self) -> Option<&InlineTable> {
        self.as_value().and_then(|v| v.as_inline_table())
    }

    pub fn as_inline_table_mut(&mut self) -> Option<&mut InlineTable> {
        self.as_value_mut().and_then(|v| v.as_inline_table_mut())
    }

    pub fn is_inline_table(&self) -> bool {
        self.as_inline_table().is_some()
    }
}

pub fn value<V: Into<Value>>(v: V) -> Item {
    Item::Value(decorated(v.into(), " ", ""))
}

pub fn table() -> Item {
    Item::Table(Table::new())
}

pub fn array() -> Item {
    Item::ArrayOfTables(ArrayOfTables::new())
}
