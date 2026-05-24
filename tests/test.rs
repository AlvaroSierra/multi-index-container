use multi_index_hashmap::multi_index_map;

#[derive(Debug, Clone, PartialEq)]
struct Person {
    email: String,
    age: u32,
    department: String,
}

multi_index_map! {
    PersonMap<Person> {
        unique email: String => |p| p.email.clone(),
        non_unique age: u32 => |p| p.age,
        non_unique department: String => |p| p.department.clone(),
    }
}

#[test]
fn test_insert_and_get() {
    let mut map = PersonMap::new();

    let person = Person {
        email: "alice@example.com".to_string(),
        age: 30,
        department: "Engineering".to_string(),
    };

    assert!(map.insert(person.clone()).is_ok());

    // Get by unique index
    assert_eq!(
        map.get_by_email(&"alice@example.com".to_string()),
        Some(&person)
    );

    // Get by non-unique index
    let by_age = map.get_by_age(&30);
    assert_eq!(by_age.len(), 1);
    assert_eq!(by_age[0], &person);
}

#[test]
fn test_unique_constraint() {
    let mut map = PersonMap::new();

    let person1 = Person {
        email: "alice@example.com".to_string(),
        age: 30,
        department: "Engineering".to_string(),
    };

    let person2 = Person {
        email: "alice@example.com".to_string(),
        age: 25,
        department: "Sales".to_string(),
    };

    assert!(map.insert(person1).is_ok());
    assert!(map.insert(person2).is_err());
}

#[test]
fn test_non_unique_index() {
    let mut map = PersonMap::new();

    let person1 = Person {
        email: "alice@example.com".to_string(),
        age: 30,
        department: "Engineering".to_string(),
    };

    let person2 = Person {
        email: "bob@example.com".to_string(),
        age: 30,
        department: "Engineering".to_string(),
    };

    assert!(map.insert(person1.clone()).is_ok());
    assert!(map.insert(person2.clone()).is_ok());

    let by_age = map.get_by_age(&30);
    assert_eq!(by_age.len(), 2);

    let by_dept = map.get_by_department(&"Engineering".to_string());
    assert_eq!(by_dept.len(), 2);
}

#[test]
fn test_remove() {
    let mut map = PersonMap::new();

    let person = Person {
        email: "alice@example.com".to_string(),
        age: 30,
        department: "Engineering".to_string(),
    };

    map.insert(person.clone()).unwrap();

    let removed = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .remove();
    assert_eq!(removed, Some(person));

    assert_eq!(map.len(), 0);
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_none());
}

#[test]
fn test_extend() {
    let mut map = PersonMap::new();

    let people = vec![
        Person {
            email: "alice@example.com".to_string(),
            age: 30,
            department: "Engineering".to_string(),
        },
        Person {
            email: "bob@example.com".to_string(),
            age: 30,
            department: "Sales".to_string(),
        },
    ];

    let errors = map.extend(people);
    assert!(errors.is_empty());
    assert_eq!(map.len(), 2);
}

#[test]
fn test_extend_with_errors() {
    let mut map = PersonMap::new();

    let people = vec![
        Person {
            email: "alice@example.com".to_string(),
            age: 30,
            department: "Engineering".to_string(),
        },
        Person {
            email: "alice@example.com".to_string(),
            age: 25,
            department: "Sales".to_string(),
        },
    ];

    let errors = map.extend(people);
    assert_eq!(errors.len(), 1);
    assert_eq!(map.len(), 1);
}
