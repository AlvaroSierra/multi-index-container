use multi_index_hashmap::multi_index_map;

#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    id: u32,
    email: String,
    age: u32,
    department: String,
}

multi_index_map! {

    /// Example documentation for the type
    pub PersonMap<Person> {
        storage_key: u32 => |p| p.id,
        unique email: String => |p| p.email.clone(),
        non_unique age: u32 => |p| p.age,
        non_unique department: String => |p| p.department.clone(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = PersonMap::new();

    let person1 = Person {
        id: 1,
        email: "alice@example.com".to_string(),
        age: 30,
        department: "Engineering".to_string(),
    };

    store.insert(person1)?;

    let person2 = Person {
        id: 2,
        email: "bob@example.com".to_string(),
        age: 30,
        department: "Engineering".to_string(),
    };

    store.insert(person2)?;

    dbg!(store.get_by_age(&30));

    Ok(())
}
