use multi_index_container::multi_index_container;

#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    email: String,
    age: u32,
    department: String,
    seniority: u32,
    team: String,
}

multi_index_container! {
    #[derive(Debug)]
    pub PersonMap<Person> {
        unique email: String => |p| p.email.clone(),
        non_unique age: u32 => |p| p.age,
        non_unique department: String => |p| p.department.clone(),
        unique_ordered seniority: u32 => |p| p.seniority,
        non_unique_ordered team: String => |p| p.team.clone(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut map = PersonMap::new();
    map.insert(Person {
        email: "alice@example.com".to_string(),
        age: 30,
        department: "engineering".to_string(),
        seniority: 5,
        team: "backend".to_string(),
    })
    .unwrap();
    map.insert(Person {
        email: "bob@example.com".to_string(),
        age: 25,
        department: "engineering".to_string(),
        seniority: 2,
        team: "backend".to_string(),
    })
    .unwrap();
    map.insert(Person {
        email: "carol@example.com".to_string(),
        age: 30,
        department: "design".to_string(),
        seniority: 7,
        team: "frontend".to_string(),
    })
    .unwrap();

    let result: Vec<_> = map.get_by_age(&30).collect();

    dbg!(result);

    Ok(())
}
