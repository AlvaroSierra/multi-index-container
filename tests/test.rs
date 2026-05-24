use multi_index_hashmap::multi_index_map;

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

fn make_map() -> PersonMap {
    let mut map = PersonMap::new();
    map.insert(Person {
        email: "alice@example.com".to_string(),
        age: 30,
        department: "engineering".to_string(),
        seniority: 5,
        team: "backend".to_string(),
    }).unwrap();
    map.insert(Person {
        email: "bob@example.com".to_string(),
        age: 25,
        department: "engineering".to_string(),
        seniority: 2,
        team: "backend".to_string(),
    }).unwrap();
    map.insert(Person {
        email: "carol@example.com".to_string(),
        age: 30,
        department: "design".to_string(),
        seniority: 7,
        team: "frontend".to_string(),
    }).unwrap();
    map
}

#[test]
fn test_remove_by_email() {
    let mut map = make_map();
    // remove via unique index
    let removed = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .remove();
    assert_eq!(removed.email, "alice@example.com");
    assert_eq!(map.len(), 2);
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_none());
    // other indexes should no longer return alice
    assert!(map.get_by_age(&30).iter().all(|p| p.email != "alice@example.com"));
    assert!(map.get_by_department(&"engineering".to_string()).iter().all(|p| p.email != "alice@example.com"));
    assert!(map.get_by_seniority(&5).is_none());
    assert!(map.get_by_team(&"backend".to_string()).iter().all(|p| p.email != "alice@example.com"));
}

#[test]
fn test_remove_by_age() {
    let mut map = make_map();

    // age=30 matches alice and carol; remove alice via email first, then verify carol remains
    let removed = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .remove();
    assert_eq!(removed.age, 30);

    // carol (age=30) must still be reachable by age
    let by_age: Vec<_> = map.get_by_age(&30);
    assert_eq!(by_age.len(), 1);
    assert_eq!(by_age[0].email, "carol@example.com");
}

#[test]
fn test_remove_by_seniority() {
    let mut map = make_map();
    // unique_ordered index: remove by seniority=7 (carol)
    let removed = map
        .get_mut_by_seniority(&7)
        .unwrap()
        .remove();
    assert_eq!(removed.email, "carol@example.com");
    assert_eq!(map.len(), 2);
    assert!(map.get_by_seniority(&7).is_none());
    assert!(map.get_by_email(&"carol@example.com".to_string()).is_none());
}

#[test]
fn test_remove_by_team() {
    let mut map = make_map();
    // non_unique_ordered: team="backend" has alice and bob; remove bob
    let removed = map
        .get_mut_by_email(&"bob@example.com".to_string())
        .unwrap()
        .remove();
    assert_eq!(removed.team, "backend");
    let backend: Vec<_> = map.get_by_team(&"backend".to_string());
    assert_eq!(backend.len(), 1);
    assert_eq!(backend[0].email, "alice@example.com");
}

#[test]
fn test_remove_by_department() {
    let mut map = make_map();
    // non_unique: department="engineering" has alice and bob; remove alice
    let removed = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .remove();
    assert_eq!(removed.department, "engineering");
    let eng: Vec<_> = map.get_by_department(&"engineering".to_string());
    assert_eq!(eng.len(), 1);
    assert_eq!(eng[0].email, "bob@example.com");
}

// --- Clash tests ---

#[test]
fn test_clash_unique_email() {
    let mut map = make_map();
    // inserting a duplicate unique email must fail
    let err = map.insert(Person {
        email: "alice@example.com".to_string(),
        age: 99,
        department: "hr".to_string(),
        seniority: 99,
        team: "other".to_string(),
    });
    assert!(err.is_err());
    assert_eq!(map.len(), 3);
}

#[test]
fn test_clash_unique_ordered_seniority() {
    let mut map = make_map();
    // inserting a duplicate unique_ordered seniority must fail
    let err = map.insert(Person {
        email: "dave@example.com".to_string(),
        age: 28,
        department: "engineering".to_string(),
        seniority: 5, // clashes with alice
        team: "backend".to_string(),
    });
    assert!(err.is_err());
    assert_eq!(map.len(), 3);
    // original alice still accessible
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_some());
    assert!(map.get_by_email(&"dave@example.com".to_string()).is_none());
}

#[test]
fn test_no_clash_non_unique_age() {
    let mut map = make_map();
    // age=30 already exists (alice, carol); a third person with age=30 is fine
    let result = map.insert(Person {
        email: "dave@example.com".to_string(),
        age: 30,
        department: "sales".to_string(),
        seniority: 10,
        team: "frontend".to_string(),
    });
    assert!(result.is_ok());
    assert_eq!(map.len(), 4);
    let by_age: Vec<_> = map.get_by_age(&30);
    assert_eq!(by_age.len(), 3);
}

#[test]
fn test_no_clash_non_unique_department() {
    let mut map = make_map();
    // department="engineering" already has two; a third is fine
    let result = map.insert(Person {
        email: "dave@example.com".to_string(),
        age: 22,
        department: "engineering".to_string(),
        seniority: 1,
        team: "devops".to_string(),
    });
    assert!(result.is_ok());
    let eng: Vec<_> = map.get_by_department(&"engineering".to_string());
    assert_eq!(eng.len(), 3);
}

#[test]
fn test_no_clash_non_unique_ordered_team() {
    let mut map = make_map();
    // team="backend" already has alice and bob; adding a third is fine
    let result = map.insert(Person {
        email: "dave@example.com".to_string(),
        age: 27,
        department: "ops".to_string(),
        seniority: 3,
        team: "backend".to_string(),
    });
    assert!(result.is_ok());
    let backend: Vec<_> = map.get_by_team(&"backend".to_string());
    assert_eq!(backend.len(), 3);
}

// --- Extend tests ---

#[test]
fn test_extend() {
    let mut map = make_map();
    let people = vec![
        Person {
            email: "dave@example.com".to_string(),
            age: 40,
            department: "sales".to_string(),
            seniority: 10,
            team: "frontend".to_string(),
        },
        Person {
            email: "eve@example.com".to_string(),
            age: 35,
            department: "design".to_string(),
            seniority: 8,
            team: "mobile".to_string(),
        },
    ];
    let errors = map.extend(people);
    assert!(errors.is_empty());
    assert_eq!(map.len(), 5);
}

#[test]
fn test_extend_clash_unique_email() {
    let mut map = make_map();
    let people = vec![
        Person {
            email: "alice@example.com".to_string(), // clashes on email
            age: 99,
            department: "hr".to_string(),
            seniority: 99,
            team: "other".to_string(),
        },
        Person {
            email: "dave@example.com".to_string(),
            age: 28,
            department: "sales".to_string(),
            seniority: 10,
            team: "mobile".to_string(),
        },
    ];
    let errors = map.extend(people);
    assert_eq!(errors.len(), 1);
    assert_eq!(map.len(), 4); // original 3 + dave
    assert!(map.get_by_email(&"dave@example.com".to_string()).is_some());
}

#[test]
fn test_extend_clash_unique_ordered_seniority() {
    let mut map = make_map();
    let people = vec![
        Person {
            email: "dave@example.com".to_string(),
            age: 28,
            department: "sales".to_string(),
            seniority: 5, // clashes with alice's seniority
            team: "mobile".to_string(),
        },
        Person {
            email: "eve@example.com".to_string(),
            age: 26,
            department: "hr".to_string(),
            seniority: 11,
            team: "mobile".to_string(),
        },
    ];
    let errors = map.extend(people);
    assert_eq!(errors.len(), 1);
    assert_eq!(map.len(), 4); // original 3 + eve
    assert!(map.get_by_email(&"eve@example.com".to_string()).is_some());
    assert!(map.get_by_email(&"dave@example.com".to_string()).is_none());
}


#[test]
fn test_get_by_unique_existing() {
    let map = make_map();
    let person = map.get_by_email(&"alice@example.com".to_string());
    assert!(person.is_some());
    assert_eq!(person.unwrap().email, "alice@example.com");
}

#[test]
fn test_get_by_unique_missing() {
    let map = make_map();
    let person = map.get_by_email(&"unknown@example.com".to_string());
    assert!(person.is_none());
}

#[test]
fn test_get_mut_by_unique_existing() {
    let mut map = make_map();
    let entry = map.get_mut_by_email(&"alice@example.com".to_string());
    assert!(entry.is_some());
}

#[test]
fn test_get_mut_by_unique_missing() {
    let mut map = make_map();
    let entry = map.get_mut_by_email(&"unknown@example.com".to_string());
    assert!(entry.is_none());
}

// --- non_unique (get_by_age, get_by_department) ---

#[test]
fn test_get_by_non_unique_multiple_results() {
    let map = make_map();
    let persons = map.get_by_age(&30);
    assert_eq!(persons.len(), 2);
    assert!(persons.iter().any(|p| p.email == "alice@example.com"));
    assert!(persons.iter().any(|p| p.email == "carol@example.com"));
}

#[test]
fn test_get_by_non_unique_single_result() {
    let map = make_map();
    let persons = map.get_by_age(&25);
    assert_eq!(persons.len(), 1);
    assert_eq!(persons[0].email, "bob@example.com");
}

#[test]
fn test_get_by_non_unique_missing() {
    let map = make_map();
    let persons = map.get_by_age(&99);
    assert!(persons.is_empty());
}

#[test]
fn test_get_mut_by_non_unique_existing() {
    let mut map = make_map();
    let entries = map.get_mut_by_department(&"engineering".to_string());
    assert!(entries.is_some());
}

#[test]
fn test_get_mut_by_non_unique_missing() {
    let mut map = make_map();
    let entries = map.get_mut_by_department(&"marketing".to_string());
    assert!(entries.is_none());
}

// --- unique_ordered (get_by_seniority) ---

#[test]
fn test_get_by_unique_ordered_existing() {
    let map = make_map();
    let person = map.get_by_seniority(&5);
    assert!(person.is_some());
    assert_eq!(person.unwrap().email, "alice@example.com");
}

#[test]
fn test_get_by_unique_ordered_missing() {
    let map = make_map();
    let person = map.get_by_seniority(&99);
    assert!(person.is_none());
}

#[test]
fn test_get_mut_by_unique_ordered_existing() {
    let mut map = make_map();
    let entry = map.get_mut_by_seniority(&5);
    assert!(entry.is_some());
}

#[test]
fn test_get_mut_by_unique_ordered_missing() {
    let mut map = make_map();
    let entry = map.get_mut_by_seniority(&99);
    assert!(entry.is_none());
}

#[test]
fn test_get_by_non_unique_ordered_multiple_results() {
    let map = make_map();
    let persons = map.get_by_team(&"backend".to_string());
    assert_eq!(persons.len(), 2);
    assert!(persons.iter().any(|p| p.email == "alice@example.com"));
    assert!(persons.iter().any(|p| p.email == "bob@example.com"));
}

#[test]
fn test_get_by_non_unique_ordered_single_result() {
    let map = make_map();
    dbg!(&map);
    let persons = map.get_by_team(&"frontend".to_string());
    assert_eq!(persons.len(), 1);
    assert_eq!(persons[0].email, "carol@example.com");
}

#[test]
fn test_get_by_non_unique_ordered_missing() {
    let map = make_map();
    let persons = map.get_by_team(&"infra".to_string());
    assert!(persons.is_empty());
}

#[test]
fn test_get_mut_by_non_unique_ordered_existing() {
    let mut map = make_map();
    let entries = map.get_mut_by_team(&"backend".to_string());
    assert!(entries.is_some());
}

#[test]
fn test_get_mut_by_non_unique_ordered_missing() {
    let mut map = make_map();
    let entries = map.get_mut_by_team(&"infra".to_string());
    assert!(entries.is_none());
}

#[test]
fn test_insert_or_overwrite_same_email() {
    let mut map = make_map();
    // Same email as alice: should replace her entry entirely
    map.insert_or_overwrite(Person {
        email: "alice@example.com".to_string(),
        age: 99,
        department: "hr".to_string(),
        seniority: 99,
        team: "other".to_string(),
    });
    assert_eq!(map.len(), 3); // count unchanged
    let alice = map.get_by_email(&"alice@example.com".to_string()).unwrap();
    assert_eq!(alice.age, 99);
    assert_eq!(alice.seniority, 99);
    // old seniority=5 must be gone from unique_ordered index
    assert!(map.get_by_seniority(&5).is_none());
    // new seniority reachable
    assert!(map.get_by_seniority(&99).is_some());
}

#[test]
fn test_insert_or_overwrite_same_seniority() {
    let mut map = make_map();
    // Same seniority as alice (5), different email: alice is evicted, new entry wins
    map.insert_or_overwrite(Person {
        email: "dave@example.com".to_string(),
        age: 28,
        department: "sales".to_string(),
        seniority: 5,
        team: "mobile".to_string(),
    });
    assert_eq!(map.len(), 3); // alice removed, dave added
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_none());
    assert!(map.get_by_email(&"dave@example.com".to_string()).is_some());
    // seniority=5 now points to dave
    assert_eq!(map.get_by_seniority(&5).unwrap().email, "dave@example.com");
    // alice's age bucket should no longer contain her
    let age_30: Vec<_> = map.get_by_age(&30);
    assert!(age_30.iter().all(|p| p.email != "alice@example.com"));
}

#[test]
fn test_insert_or_overwrite_no_clash() {
    let mut map = make_map();
    map.insert_or_overwrite(Person {
        email: "dave@example.com".to_string(),
        age: 28,
        department: "sales".to_string(),
        seniority: 10,
        team: "mobile".to_string(),
    });
    assert_eq!(map.len(), 4);
    assert!(map.get_by_email(&"dave@example.com".to_string()).is_some());
    assert!(map.get_by_seniority(&10).is_some());
    let sales: Vec<_> = map.get_by_department(&"sales".to_string());
    assert_eq!(sales.len(), 1);
}

#[test]
fn test_insert_or_overwrite_clears_all_indexes_of_evicted_entry() {
    let mut map = make_map();
    // Overwrite bob (age=25, department=engineering, seniority=2, team=backend)
    // by clashing on his email
    map.insert_or_overwrite(Person {
        email: "bob@example.com".to_string(),
        age: 50,
        department: "legal".to_string(),
        seniority: 20,
        team: "compliance".to_string(),
    });
    assert_eq!(map.len(), 3);

    // bob's OLD values must be gone from every index
    let age_25: Vec<_> = map.get_by_age(&25);
    assert!(age_25.is_empty());

    let eng: Vec<_> = map.get_by_department(&"engineering".to_string());
    assert!(eng.iter().all(|p| p.email != "bob@example.com"));

    assert!(map.get_by_seniority(&2).is_none());

    let backend: Vec<_> = map.get_by_team(&"backend".to_string());
    assert!(backend.iter().all(|p| p.email != "bob@example.com"));

    // bob's NEW values must be reachable from every index
    let age_50: Vec<_> = map.get_by_age(&50);
    assert_eq!(age_50.len(), 1);
    assert_eq!(age_50[0].email, "bob@example.com");

    assert_eq!(map.get_by_seniority(&20).unwrap().email, "bob@example.com");

    let legal: Vec<_> = map.get_by_department(&"legal".to_string());
    assert_eq!(legal.len(), 1);

    let compliance: Vec<_> = map.get_by_team(&"compliance".to_string());
    assert_eq!(compliance.len(), 1);
}

#[test]
fn test_insert_or_overwrite_multi_clash_removes_all() {
    let mut map = make_map();
    // email clashes with alice, seniority clashes with bob (seniority=2)
    // Both should be evicted; only the new entry and carol remain
    map.insert_or_overwrite(Person {
        email: "alice@example.com".to_string(),
        age: 40,
        department: "product".to_string(),
        seniority: 2, // bob's seniority
        team: "strategy".to_string(),
    });
    assert_eq!(map.len(), 2); // carol + new entry

    assert!(map.get_by_email(&"bob@example.com".to_string()).is_none());
    assert!(map.get_by_seniority(&5).is_none()); // alice's old seniority gone

    let new_alice = map.get_by_email(&"alice@example.com".to_string()).unwrap();
    assert_eq!(new_alice.seniority, 2);
    assert_eq!(new_alice.department, "product");
}

#[test]
fn test_insert_or_overwrite_idempotent() {
    let mut map = make_map();
    let carol = map.get_by_email(&"carol@example.com".to_string()).unwrap().clone();
    map.insert_or_overwrite(carol.clone());
    assert_eq!(map.len(), 3);
    let fetched = map.get_by_email(&"carol@example.com".to_string()).unwrap();
    assert_eq!(*fetched, carol);
    assert_eq!(map.get_by_seniority(&7).unwrap().email, "carol@example.com");
    let frontend: Vec<_> = map.get_by_team(&"frontend".to_string());
    assert_eq!(frontend.len(), 1);
}
