use multi_index_container::multi_index_container;

#[derive(Debug, Clone, PartialEq)]
struct Person {
    email: String,
    age: u32,
    department: String,
    seniority: u32,
    team: String,
}

multi_index_container! {
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
    assert!(
        map.get_by_age(&30)
            .iter()
            .all(|p| p.email != "alice@example.com")
    );
    assert!(
        map.get_by_department(&"engineering".to_string())
            .iter()
            .all(|p| p.email != "alice@example.com")
    );
    assert!(map.get_by_seniority(&5).is_none());
    assert!(
        map.get_by_team(&"backend".to_string())
            .iter()
            .all(|p| p.email != "alice@example.com")
    );
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
    let removed = map.get_mut_by_seniority(&7).unwrap().remove();
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
    let carol = map
        .get_by_email(&"carol@example.com".to_string())
        .unwrap()
        .clone();
    map.insert_or_overwrite(carol.clone());
    assert_eq!(map.len(), 3);
    let fetched = map.get_by_email(&"carol@example.com".to_string()).unwrap();
    assert_eq!(*fetched, carol);
    assert_eq!(map.get_by_seniority(&7).unwrap().email, "carol@example.com");
    let frontend: Vec<_> = map.get_by_team(&"frontend".to_string());
    assert_eq!(frontend.len(), 1);
}

#[test]
fn test_get_by_department_team_match() {
    let map = make_map();
    // alice and bob are both engineering/backend
    let results: Vec<_> =
        map.get_by_department_team(&"engineering".to_string(), &"backend".to_string());
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|p| p.email == "alice@example.com"));
    assert!(results.iter().any(|p| p.email == "bob@example.com"));
}

#[test]
fn test_get_by_department_team_no_match() {
    let map = make_map();
    // no one is in engineering/frontend
    let results: Vec<_> =
        map.get_by_department_team(&"engineering".to_string(), &"frontend".to_string());
    assert!(results.is_empty());
}

#[test]
fn test_get_by_department_team_single_match() {
    let map = make_map();
    // only carol is design/frontend
    let results: Vec<_> =
        map.get_by_department_team(&"design".to_string(), &"frontend".to_string());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].email, "carol@example.com");
}

#[test]
fn test_get_by_department_team_unknown_department() {
    let map = make_map();
    let results: Vec<_> = map.get_by_department_team(&"hr".to_string(), &"backend".to_string());
    assert!(results.is_empty());
}

#[test]
fn test_get_by_department_team_unknown_team() {
    let map = make_map();
    let results: Vec<_> =
        map.get_by_department_team(&"engineering".to_string(), &"mobile".to_string());
    assert!(results.is_empty());
}

// --- get_by_age_department_team ---

#[test]
fn test_get_by_age_department_team_single_match() {
    let map = make_map();
    // alice: age=30, engineering, backend
    let results: Vec<_> =
        map.get_by_age_department_team(&30, &"engineering".to_string(), &"backend".to_string());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].email, "alice@example.com");
}

#[test]
fn test_get_by_age_department_team_age_narrows_department_team() {
    let map = make_map();
    // bob is also engineering/backend but age=25, not 30; must not appear
    let results: Vec<_> =
        map.get_by_age_department_team(&30, &"engineering".to_string(), &"backend".to_string());
    assert!(results.iter().all(|p| p.email != "bob@example.com"));
}

#[test]
fn test_get_by_age_department_team_no_match_wrong_age() {
    let map = make_map();
    // engineering/backend exists, but not at age=99
    let results: Vec<_> =
        map.get_by_age_department_team(&99, &"engineering".to_string(), &"backend".to_string());
    assert!(results.is_empty());
}

#[test]
fn test_get_by_age_department_team_no_match_wrong_department() {
    let map = make_map();
    // age=30 and team=backend exist, but not with department=sales
    let results: Vec<_> =
        map.get_by_age_department_team(&30, &"sales".to_string(), &"backend".to_string());
    assert!(results.is_empty());
}

#[test]
fn test_get_by_age_department_team_no_match_wrong_team() {
    let map = make_map();
    // age=30 and department=engineering exist, but not with team=mobile
    let results: Vec<_> =
        map.get_by_age_department_team(&30, &"engineering".to_string(), &"mobile".to_string());
    assert!(results.is_empty());
}

#[test]
fn test_get_by_age_department_team_all_unknown() {
    let map = make_map();
    let results: Vec<_> =
        map.get_by_age_department_team(&0, &"unknown".to_string(), &"unknown".to_string());
    assert!(results.is_empty());
}

// --- cross-check: get_by_department_team vs get_by_age_department_team ---

#[test]
fn test_combined_subset_of_department_team() {
    let map = make_map();
    // get_by_age_department_team results must always be a subset of get_by_department_team
    let by_dept_team: Vec<_> = map
        .get_by_department_team(&"engineering".to_string(), &"backend".to_string())
        .into_iter()
        .map(|p| p.email.clone())
        .collect();
    let by_age_dept_team: Vec<_> = map
        .get_by_age_department_team(&30, &"engineering".to_string(), &"backend".to_string())
        .into_iter()
        .map(|p| p.email.clone())
        .collect();
    for email in &by_age_dept_team {
        assert!(by_dept_team.contains(email));
    }
}

// --- get_mut_by_department_team ---
#[test]
fn test_remove_via_get_mut_by_department_team_single_match() {
    let mut map = make_map();
    // design/frontend only has carol; remove her
    let removed = map
        .get_mut_by_department_team(&"design".to_string(), &"frontend".to_string())
        .unwrap()
        .first()
        .unwrap()
        .remove();
    assert_eq!(removed.email, "carol@example.com");
    assert_eq!(map.len(), 2);
    // gone from all indexes
    assert!(map.get_by_email(&"carol@example.com".to_string()).is_none());
    assert!(map.get_by_seniority(&7).is_none());
    let age_30: Vec<_> = map.get_by_age(&30);
    assert!(age_30.iter().all(|p| p.email != "carol@example.com"));
    let frontend: Vec<_> = map.get_by_team(&"frontend".to_string());
    assert!(frontend.is_empty());
    let design: Vec<_> = map.get_by_department(&"design".to_string());
    assert!(design.is_empty());
}

#[test]
fn test_remove_via_get_mut_by_department_team_one_of_many() {
    let mut map = make_map();
    // engineering/backend has alice and bob; remove only the first result
    let removed = map
        .get_mut_by_department_team(&"engineering".to_string(), &"backend".to_string())
        .unwrap()
        .first()
        .unwrap()
        .remove();
    assert_eq!(map.len(), 2);
    // the other engineering/backend person must still be reachable via the combined index
    let remaining: Vec<_> = map
        .get_by_department_team(&"engineering".to_string(), &"backend".to_string());
    assert_eq!(remaining.len(), 1);
    assert_ne!(remaining[0].email, removed.email);
}

#[test]
fn test_remove_all_via_get_mut_by_department_team() {
    let mut map = make_map();
    // drain all engineering/backend entries
    let emails: Vec<String> = map
        .get_mut_by_department_team(&"engineering".to_string(), &"backend".to_string())
        .unwrap()
        .remove_all()
        .into_iter()
        .map(|x| x.email)
        .collect()
        ;
    assert_eq!(emails.len(), 2);
    assert!(emails.contains(&"alice@example.com".to_string()));
    assert!(emails.contains(&"bob@example.com".to_string()));
    assert_eq!(map.len(), 1);
    // combined index must now be empty
    let remaining: Vec<_> = map
        .get_by_department_team(&"engineering".to_string(), &"backend".to_string());
    assert!(remaining.is_empty());
    // carol is untouched
    assert!(map.get_by_email(&"carol@example.com".to_string()).is_some());
}

// --- get_mut_by_age_department_team ---

#[test]
fn test_remove_via_get_mut_by_age_department_team_single_match() {
    let mut map = make_map();
    // age=30, engineering, backend matches only alice (bob is age=25)
    let removed = map
        .get_mut_by_age_department_team(&30, &"engineering".to_string(), &"backend".to_string())
        .unwrap()
        .first()
        .unwrap()
        .remove();
    assert_eq!(removed.email, "alice@example.com");
    assert_eq!(map.len(), 2);
    // gone from all indexes
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_none());
    assert!(map.get_by_seniority(&5).is_none());
    // bob (age=25, engineering, backend) must be unaffected
    assert!(map.get_by_email(&"bob@example.com".to_string()).is_some());
    let eng_backend: Vec<_> = map
        .get_by_department_team(&"engineering".to_string(), &"backend".to_string());
    assert_eq!(eng_backend.len(), 1);
    assert_eq!(eng_backend[0].email, "bob@example.com");
}

#[test]
fn test_remove_via_get_mut_by_age_department_team_no_match() {
    let mut map = make_map();
    // age=99 matches nobody; iterator is empty, map unchanged
    let iter =
        map.get_mut_by_age_department_team(&99, &"engineering".to_string(), &"backend".to_string());
    assert!(iter.is_none());
    assert_eq!(map.len(), 3);
}

#[test]
fn test_remove_via_get_mut_by_age_department_team_clears_all_indexes() {
    let mut map = make_map();
    // remove bob via age=25, engineering, backend
    let removed = map
        .get_mut_by_age_department_team(&25, &"engineering".to_string(), &"backend".to_string())
        .unwrap()
        .first()
        .unwrap()
        .remove();
    assert_eq!(removed.email, "bob@example.com");
    assert_eq!(map.len(), 2);
    // bob gone from every index
    assert!(map.get_by_seniority(&2).is_none());
    let age_25: Vec<_> = map.get_by_age(&25);
    assert!(age_25.is_empty());
    let eng: Vec<_> = map.get_by_department(&"engineering".to_string());
    assert!(eng.iter().all(|p| p.email != "bob@example.com"));
    let backend: Vec<_> = map.get_by_team(&"backend".to_string());
    assert!(backend.iter().all(|p| p.email != "bob@example.com"));
    // alice still reachable via the same combined index
    let remaining: Vec<_> = map
        .get_by_age_department_team(&30, &"engineering".to_string(), &"backend".to_string())
        .into_iter()
        .map(|e| e.email.clone())
        .collect();
    assert_eq!(remaining, vec!["alice@example.com".to_string()]);
}

// --- modify_or_remove via get_mut_by_email (unique index) ---

#[test]
fn test_modify_non_indexed_field() {
    let mut map = make_map();
    // modifying a field that isn't part of any index is a simple in-place update
    let result = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .modify_or_remove(|p| p.age = 99);
    assert!(result.is_ok());
    assert_eq!(map.len(), 3);
    let alice = map.get_by_email(&"alice@example.com".to_string()).unwrap();
    assert_eq!(alice.age, 99);
}

#[test]
fn test_modify_unique_index_to_unused_value() {
    let mut map = make_map();
    // changing email to a value not held by anyone else succeeds
    let result = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .modify_or_remove(|p| p.email = "alice-new@example.com".to_string());
    assert!(result.is_ok());
    assert_eq!(map.len(), 3);
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_none());
    assert!(
        map.get_by_email(&"alice-new@example.com".to_string())
            .is_some()
    );
}

#[test]
fn test_modify_unique_index_clash_removes_entry() {
    let mut map = make_map();
    // changing alice's email to bob's causes a clash on re-insert;
    // alice is permanently removed (not rolled back)
    let result = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .modify_or_remove(|p| p.email = "bob@example.com".to_string());
    assert!(result.is_err());
    assert_eq!(map.len(), 2); // alice gone, bob untouched
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_none());
    assert!(map.get_by_email(&"bob@example.com".to_string()).is_some());
    // alice's old indexed values must also be gone
    assert!(map.get_by_seniority(&5).is_none());
    let age_30: Vec<_> = map.get_by_age(&30);
    assert!(age_30.iter().all(|p| p.email != "alice@example.com"));
    let backend: Vec<_> = map.get_by_team(&"backend".to_string());
    assert!(backend.iter().all(|p| p.email != "alice@example.com"));
}

#[test]
fn test_modify_unique_ordered_index_to_unused_value() {
    let mut map = make_map();
    // changing alice's seniority to a value no one else holds succeeds
    let result = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .modify_or_remove(|p| p.seniority = 50);
    assert!(result.is_ok());
    assert_eq!(map.len(), 3);
    assert!(map.get_by_seniority(&5).is_none());
    assert_eq!(
        map.get_by_seniority(&50).unwrap().email,
        "alice@example.com"
    );
}

#[test]
fn test_modify_unique_ordered_index_clash_removes_entry() {
    let mut map = make_map();
    // changing alice's seniority to carol's (7) causes a clash; alice is lost
    let result = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .modify_or_remove(|p| p.seniority = 7);
    assert!(result.is_err());
    assert_eq!(map.len(), 2);
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_none());
    // carol's seniority=7 still points to carol
    assert_eq!(map.get_by_seniority(&7).unwrap().email, "carol@example.com");
    assert!(map.get_by_seniority(&5).is_none());
}

#[test]
fn test_modify_non_unique_index_no_clash() {
    let mut map = make_map();
    // moving alice from engineering to a new department; no clash possible on non-unique index
    let result = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .modify_or_remove(|p| p.department = "product".to_string());
    assert!(result.is_ok());
    assert_eq!(map.len(), 3);
    // alice no longer in engineering bucket
    let eng: Vec<_> = map.get_by_department(&"engineering".to_string());
    assert!(eng.iter().all(|p| p.email != "alice@example.com"));
    // alice now in product bucket
    let product: Vec<_> = map.get_by_department(&"product".to_string());
    assert_eq!(product.len(), 1);
    assert_eq!(product[0].email, "alice@example.com");
}

#[test]
fn test_modify_non_unique_ordered_index_no_clash() {
    let mut map = make_map();
    // moving alice from backend to a new team; non_unique_ordered allows duplicates
    let result = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .modify_or_remove(|p| p.team = "mobile".to_string());
    assert!(result.is_ok());
    assert_eq!(map.len(), 3);
    let backend: Vec<_> = map.get_by_team(&"backend".to_string());
    assert!(backend.iter().all(|p| p.email != "alice@example.com"));
    assert_eq!(backend.len(), 1); // only bob remains
    let mobile: Vec<_> = map.get_by_team(&"mobile".to_string());
    assert_eq!(mobile.len(), 1);
    assert_eq!(mobile[0].email, "alice@example.com");
}

// --- modify_or_remove via combined get_mut ---

#[test]
fn test_modify_via_get_mut_by_department_team() {
    let mut map = make_map();
    // promote alice's seniority via the combined index
    let result = map
        .get_mut_by_department_team(&"engineering".to_string(), &"backend".to_string())
        .unwrap()
        .find(|e| e.email == "alice@example.com")
        .unwrap()
        .modify_or_remove(|p| p.seniority = 99);
    assert!(result.is_ok());
    assert_eq!(map.len(), 3);
    assert_eq!(
        map.get_by_seniority(&99).unwrap().email,
        "alice@example.com"
    );
    assert!(map.get_by_seniority(&5).is_none());
}

#[test]
fn test_modify_via_get_mut_by_department_team_clash_removes_entry() {
    let mut map = make_map();
    // changing alice's seniority to bob's (2) via combined index; alice is lost on clash
    let result = map
        .get_mut_by_department_team(&"engineering".to_string(), &"backend".to_string())
        .unwrap()
        .find(|e| e.email == "alice@example.com")
        .unwrap()
        .modify_or_remove(|p| p.seniority = 2);
    assert!(result.is_err());
    assert_eq!(map.len(), 2);
    assert!(map.get_by_email(&"alice@example.com".to_string()).is_none());
    // bob's seniority=2 still intact
    assert_eq!(map.get_by_seniority(&2).unwrap().email, "bob@example.com");
}

#[test]
fn test_modify_via_get_mut_by_age_department_team() {
    let mut map = make_map();
    // update alice's team via the three-field combined index
    let result = map
        .get_mut_by_age_department_team(&30, &"engineering".to_string(), &"backend".to_string())
        .unwrap()
        .first()
        .unwrap()
        .modify_or_remove(|p| p.team = "platform".to_string());
    assert!(result.is_ok());
    assert_eq!(map.len(), 3);
    // alice no longer in backend bucket
    let backend: Vec<_> = map.get_by_team(&"backend".to_string());
    assert!(backend.iter().all(|p| p.email != "alice@example.com"));
    // alice now in platform bucket
    let platform: Vec<_> = map.get_by_team(&"platform".to_string());
    assert_eq!(platform.len(), 1);
    assert_eq!(platform[0].email, "alice@example.com");
    // alice no longer reachable via old combined index
    let old_combined: Vec<_> = map
        .get_by_age_department_team(&30, &"engineering".to_string(), &"backend".to_string());
    assert!(old_combined.iter().all(|p| p.email != "alice@example.com"));
}

// --- error value contains the orphaned entry ---

#[test]
fn test_modify_clash_error_contains_attempted_value() {
    let mut map = make_map();
    // the Err should carry back the value that failed to re-insert
    let result = map
        .get_mut_by_email(&"alice@example.com".to_string())
        .unwrap()
        .modify_or_remove(|p| {
            p.email = "bob@example.com".to_string();
            p.seniority = 999;
        });
    let err = result.unwrap_err();
    // the orphaned value reflects the mutation that was attempted
    assert_eq!(err.value.email, "bob@example.com");
    assert_eq!(err.value.seniority, 999);
}
