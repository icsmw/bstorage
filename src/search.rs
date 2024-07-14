use crate::{Storage, E};
use serde::Deserialize;

/// The `Search` trait provides methods for searching records in the storage.
pub trait Search {
    /// Finds the first record that matches the specified condition.
    ///
    /// # Arguments
    ///
    /// * `condition` - A closure that takes a reference to a value and returns a boolean indicating if the value matches the condition.
    ///
    /// # Returns
    ///
    /// * `Result<Option<V>, E>` - Returns the first matching value if found, or None if no match is found, or an error.
    fn find<V: for<'a> Deserialize<'a> + 'static, F: Fn(&V) -> bool>(
        &self,
        condition: F,
    ) -> Result<Option<V>, E>;

    /// Filters the records and returns all that match the specified condition.
    ///
    /// # Arguments
    ///
    /// * `condition` - A closure that takes a reference to a value and returns a boolean indicating if the value matches the condition.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<V>, E>` - Returns a vector of all matching values, or an error.
    fn filter<V: for<'a> Deserialize<'a> + 'static, F: Fn(&V) -> bool>(
        &self,
        condition: F,
    ) -> Result<Vec<V>, E>;
}

impl Search for Storage {
    /// Finds the first record in the storage that matches the specified condition.
    ///
    /// # Arguments
    ///
    /// * `condition` - A closure that takes a reference to a value and returns a boolean indicating if the value matches the condition.
    ///
    /// # Returns
    ///
    /// * `Result<Option<V>, E>` - Returns the first matching value if found, or None if no match is found, or an error.
    ///
    /// # Example
    /// ```
    /// use bstorage::{Search, Storage, E};
    /// use serde::{Deserialize, Serialize};
    /// use std::{env::temp_dir, fs::remove_dir_all};
    /// use uuid::Uuid;
    ///
    /// #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    /// struct A {
    ///     a: u8,
    ///     b: String,
    /// }
    ///
    /// #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    /// struct B {
    ///     c: u32,
    ///     d: Option<bool>,
    /// }
    ///
    /// let mut storage = Storage::create(temp_dir().join(Uuid::new_v4().to_string())).unwrap();
    /// let a = [
    ///     A {
    ///         a: 0,
    ///         b: String::from("one"),
    ///     },
    ///     A {
    ///         a: 1,
    ///         b: String::from("two"),
    ///     },
    ///     A {
    ///         a: 2,
    ///         b: String::from("three"),
    ///     },
    /// ];
    /// let b = [
    ///     B {
    ///         c: 0,
    ///         d: Some(true),
    ///     },
    ///     B {
    ///         c: 1,
    ///         d: Some(false),
    ///     },
    ///     B {
    ///         c: 2,
    ///         d: Some(true),
    ///     },
    /// ];
    /// let mut i = 0;
    /// for a in a.iter() {
    ///     storage.set(i.to_string(), a).unwrap();
    ///     i += 1;
    /// }
    /// for b in b.iter() {
    ///     storage.set(i.to_string(), b).unwrap();
    ///     i += 1;
    /// }
    /// let found = storage.find(|v: &A| &a[0] == v).unwrap().expect("Record found");
    /// assert_eq!(found, a[found.a as usize]);
    /// let found = storage.find(|v: &B| &b[0] == v).unwrap().expect("Record found");
    /// assert_eq!(found, b[found.c as usize]);
    /// assert!(storage.find(|v: &A| v.a > 254).unwrap().is_none());
    /// storage.clear().unwrap();
    /// remove_dir_all(storage.cwd()).unwrap();
    /// ```
    fn find<V: for<'a> Deserialize<'a> + 'static, F: Fn(&V) -> bool>(
        &self,
        condition: F,
    ) -> Result<Option<V>, E> {
        for key in self.into_iter() {
            let Some(v) = self.get::<V, &String>(key)? else {
                continue;
            };
            if condition(&v) {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }

    /// Filters the records in the storage and returns all that match the specified condition.
    ///
    /// # Arguments
    ///
    /// * `condition` - A closure that takes a reference to a value and returns a boolean indicating if the value matches the condition.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<V>, E>` - Returns a vector of all matching values, or an error.
    ///
    /// # Example
    ///
    ///```
    /// use bstorage::{Search, Storage, E};
    /// use serde::{Deserialize, Serialize};
    /// use std::{env::temp_dir, fs::remove_dir_all};
    /// use uuid::Uuid;
    ///
    /// #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    /// struct A {
    ///     a: u8,
    ///     b: String,
    /// }
    ///
    /// #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    /// struct B {
    ///     c: u32,
    ///     d: Option<bool>,
    /// }
    ///
    /// let mut storage = Storage::create(temp_dir().join(Uuid::new_v4().to_string())).unwrap();
    /// let a = [
    ///     A {
    ///         a: 0,
    ///         b: String::from("one"),
    ///     },
    ///     A {
    ///         a: 1,
    ///         b: String::from("two"),
    ///     },
    ///     A {
    ///         a: 2,
    ///         b: String::from("three"),
    ///     },
    /// ];
    /// let b = [
    ///     B {
    ///         c: 0,
    ///         d: Some(true),
    ///     },
    ///     B {
    ///         c: 1,
    ///         d: Some(false),
    ///     },
    ///     B {
    ///         c: 2,
    ///         d: Some(true),
    ///     },
    /// ];
    /// let mut i = 0;
    /// for a in a.iter() {
    ///     storage.set(i.to_string(), a).unwrap();
    ///     i += 1;
    /// }
    /// for b in b.iter() {
    ///     storage.set(i.to_string(), b).unwrap();
    ///     i += 1;
    /// }
    /// let found = storage.filter(|v: &A| v.a < 2).unwrap();
    /// assert_eq!(found.len(), 2);
    /// for found in found.into_iter() {
    ///     assert_eq!(found, a[found.a as usize]);
    /// }
    /// let found = storage.filter(|v: &B| v.c < 2).unwrap();
    /// assert_eq!(found.len(), 2);
    /// for found in found.into_iter() {
    ///     assert_eq!(found, b[found.c as usize]);
    /// }
    /// assert_eq!(storage.filter(|v: &A| v.a > 254).unwrap().len(), 0);
    /// storage.clear().unwrap();
    /// remove_dir_all(storage.cwd()).unwrap();
    /// ```
    fn filter<V: for<'a> Deserialize<'a> + 'static, F: Fn(&V) -> bool>(
        &self,
        condition: F,
    ) -> Result<Vec<V>, E> {
        let mut filtered = Vec::new();
        for key in self.into_iter() {
            let Some(v) = self.get::<V, &String>(key)? else {
                continue;
            };
            if condition(&v) {
                filtered.push(v);
            }
        }
        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Search, Storage, E};
    use serde::{Deserialize, Serialize};
    use std::{env::temp_dir, fs::remove_dir_all};
    use uuid::Uuid;

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    struct A {
        a: u8,
        b: String,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    struct B {
        c: u32,
        d: Option<bool>,
    }

    #[test]
    fn find() -> Result<(), E> {
        let mut storage = Storage::create(temp_dir().join(Uuid::new_v4().to_string()))?;
        let a = [
            A {
                a: 0,
                b: String::from("one"),
            },
            A {
                a: 1,
                b: String::from("two"),
            },
            A {
                a: 2,
                b: String::from("three"),
            },
        ];
        let b = [
            B {
                c: 0,
                d: Some(true),
            },
            B {
                c: 1,
                d: Some(false),
            },
            B {
                c: 2,
                d: Some(true),
            },
        ];
        let mut i = 0;
        for a in a.iter() {
            storage.set(i.to_string(), a)?;
            i += 1;
        }
        for b in b.iter() {
            storage.set(i.to_string(), b)?;
            i += 1;
        }
        let found = storage.find(|v: &A| &a[0] == v)?.expect("Record found");
        assert_eq!(found, a[found.a as usize]);
        let found = storage.find(|v: &B| &b[0] == v)?.expect("Record found");
        assert_eq!(found, b[found.c as usize]);
        assert!(storage.find(|v: &A| v.a > 254)?.is_none());
        storage.clear()?;
        remove_dir_all(storage.cwd())?;
        Ok(())
    }

    #[test]
    fn filter() -> Result<(), E> {
        let mut storage = Storage::create(temp_dir().join(Uuid::new_v4().to_string()))?;
        let a = [
            A {
                a: 0,
                b: String::from("one"),
            },
            A {
                a: 1,
                b: String::from("two"),
            },
            A {
                a: 2,
                b: String::from("three"),
            },
        ];
        let b = [
            B {
                c: 0,
                d: Some(true),
            },
            B {
                c: 1,
                d: Some(false),
            },
            B {
                c: 2,
                d: Some(true),
            },
        ];
        let mut i = 0;
        for a in a.iter() {
            storage.set(i.to_string(), a)?;
            i += 1;
        }
        for b in b.iter() {
            storage.set(i.to_string(), b)?;
            i += 1;
        }
        let found = storage.filter(|v: &A| v.a < 2)?;
        assert_eq!(found.len(), 2);
        for found in found.into_iter() {
            assert_eq!(found, a[found.a as usize]);
        }
        let found = storage.filter(|v: &B| v.c < 2)?;
        assert_eq!(found.len(), 2);
        for found in found.into_iter() {
            assert_eq!(found, b[found.c as usize]);
        }
        assert_eq!(storage.filter(|v: &A| v.a > 254)?.len(), 0);
        storage.clear()?;
        remove_dir_all(storage.cwd())?;
        Ok(())
    }
}
