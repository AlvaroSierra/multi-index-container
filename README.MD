# multi_index_container

A Rust procedural macro for collections that can be efficiently looked up, mutated, and removed by multiple indexes simultaneously, with compile-time guarantees that unique constraints are upheld.

## Overview

`multi_index_container` lets you define a typed map over any struct where fields can be designated as indexes. Each index can be unique or non-unique, ordered or unordered. The macro generates all lookup, mutation, and removal methods at compile time with no runtime overhead from reflection or dynamic dispatch.

## Index Types

| Index type | Duplicates | Ordered |
|---|---|---|
| `unique` | No | No |
| `non_unique` | Yes | No |
| `unique_ordered` | No | Yes |
| `non_unique_ordered` | Yes | Yes |

## Usage

```rust
use multi_index_map::multi_index_map;

#[derive(Debug, Clone, PartialEq)]
struct Person {
    email: String,
    age: u32,
    department: String,
    seniority: u32,
    team: String,
}

multi_index_map! {
    #[derive(Debug)]
    PersonMap<Person> {
        unique email: String => |p| p.email.clone(),
        non_unique age: u32 => |p| p.age,
        non_unique department: String => |p| p.department.clone(),
        unique_ordered seniority: u32 => |p| p.seniority,
        non_unique_ordered team: String => |p| p.team.clone(),
    }
}
```

This generates a `PersonMap` type with methods for every index and every combination of indexes.

### Documentation

This macro will also generate documentation for the types and methods generated which can be read by generating documenation `cargo doc --open --no-deps`.

## Inserting

```rust
let mut map = PersonMap::new();

// Returns Err if a unique constraint is violated
map.insert(Person { email: "alice@example.com".into(), age: 30, .. }).unwrap();

// Overwrites any entry that clashes on a unique index; all clashing entries are removed
map.insert_or_overwrite(Person { email: "alice@example.com".into(), age: 99, .. });

// Extend from an iterator; returns a Vec of values that failed to insert
let errors = map.extend(people);
```

## Looking Up

Single-index lookups are generated for every field:

```rust
// Unique indexes return Option<&T>
let alice = map.get_by_email(&"alice@example.com".to_string());

// Non-unique indexes return impl Iterator<Item = &T>
let engineers: Vec<_> = map.get_by_department(&"engineering".to_string()).collect();

// Ordered indexes support range queries
let seniors: Vec<_> = map.get_by_seniority_range(5..).collect();
```

Combined lookups are generated for every combination of non-unique indexes:

```rust
let results: Vec<_> = map
    .get_by_department_team(&"engineering".to_string(), &"backend".to_string())
    .collect();

let results: Vec<_> = map
    .get_by_age_department_team(&30, &"engineering".to_string(), &"backend".to_string())
    .collect();
```

## Mutating and Removing

`get_mut_by_*` returns a `MutEntries` handle that supports chained operations before committing:

```rust
// Filter, then modify the first match
map.get_mut_by_department_team(&"engineering".to_string(), &"backend".to_string())
    .filter(|p| p.age > 25)
    .first()
    .unwrap()
    .modify_or_remove(|p| p.seniority += 1)
    .unwrap();

// Remove a specific entry
let removed = map
    .get_mut_by_email(&"alice@example.com".to_string())
    .unwrap()
    .remove();

// Remove all matching entries, getting back the evicted values
let removed: Vec<Person> = map
    .get_mut_by_team(&"backend".to_string())
    .remove_all();
```

### MutEntries API

| Method | Description |
|---|---|
| `filter(fn)` | Narrow entries by predicate (read-only pass) |
| `find(fn)` | Return first entry matching predicate as `Option<MutEntry>` |
| `first()` | Return the first entry as `Option<MutEntry>` |
| `last()` | Return the last entry as `Option<MutEntry>` |
| `nth(n)` | Return the nth entry as `Option<MutEntry>` |
| `count()` | Number of matched entries |
| `is_empty()` | Whether any entries matched |
| `collect_values()` | Read-only `Vec<&T>` of all matched entries |
| `remove_all()` | Remove and return all matched entries as `Vec<T>` |
| `for_each(fn)` | Apply a closure to each entry in turn |

### modify_or_remove

`modify_or_remove` removes the entry, applies your closure, then re-inserts. If re-insertion fails due to a unique constraint clash, the entry is permanently removed and the failed value is returned in the `Err`:

```rust
let result = map
    .get_mut_by_email(&"alice@example.com".to_string())
    .unwrap()
    .modify_or_remove(|p| p.seniority = 10);

match result {
    Ok(()) => { /* updated successfully */ }
    Err(e) => { /* e.value is the orphaned Person */ }
}
```

## Unique Constraint Violations

Any insertion that would violate a `unique` or `unique_ordered` index returns an `Err` containing the value that could not be inserted. `insert_or_overwrite` instead removes all clashing entries, including across multiple indexes, and inserts the new value unconditionally.
