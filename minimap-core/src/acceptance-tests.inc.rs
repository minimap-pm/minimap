// This is NOT a complete rust file; it's a snippet that should
// be included into test modules for workspaces.
//
// Define your own `create_test_workspace!()` macro that returns a
// `T: workspace` of some sort, and then include it:
//
// ```rust
// macro_rules! create_test_workspace {
//     () => (Myworkspace::new())
// }
//
// include!("acceptance-tests.inc.rs");
// ```

#[test]
fn test_commit() {
	let workspace = create_test_workspace!();

	let commit = workspace.record_builder("coll").commit("test").unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.get_record(&Record::id(&commit)).unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test");
}

#[test]
fn test_walk() {
	let workspace = create_test_workspace!();

	let first = workspace.record_builder("coll").commit("test").unwrap();
	let second = workspace.record_builder("coll").commit("test2").unwrap();

	let mut iter = workspace.walk("coll").unwrap();
	let commit = iter.next().unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test2");
	assert_eq!(Record::id(&commit), second.id().to_string());
	let commit = iter.next().unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(Record::id(&commit), first.id().to_string());
	assert!(iter.next().is_none());
}

#[test]
fn test_set_walk() {
	let workspace = create_test_workspace!();

	workspace.set_add_unchecked("coll", "test").unwrap();
	workspace.set_del_unchecked("coll", "test").unwrap();
	workspace.set_add_unchecked("coll", "test2").unwrap();
	workspace.set_add_unchecked("coll", "test3").unwrap();
	workspace.set_add_unchecked("coll", "test4").unwrap();
	workspace.set_add_unchecked("coll", "test5").unwrap();
	workspace.set_del_unchecked("coll", "test4").unwrap();

	let records = workspace
		.walk_set("coll")
		.unwrap()
		.map(|r| r.unwrap())
		.map(|(record, op)| (Record::message(&record), op))
		.collect::<Vec<_>>();
	assert_eq!(records.len(), 7);
	assert_eq!(records[6], ("test".to_string(), SetOperation::Add));
	assert_eq!(records[5], ("test".to_string(), SetOperation::Del));
	assert_eq!(records[4], ("test2".to_string(), SetOperation::Add));
	assert_eq!(records[3], ("test3".to_string(), SetOperation::Add));
	assert_eq!(records[2], ("test4".to_string(), SetOperation::Add));
	assert_eq!(records[1], ("test5".to_string(), SetOperation::Add));
	assert_eq!(records[0], ("test4".to_string(), SetOperation::Del));
}

#[test]
fn test_set() {
	let workspace = create_test_workspace!();

	let commit = workspace.set_add_unchecked("coll", "test").unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.set_find("coll", "test").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.set_del_unchecked("coll", "test").unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace
		.set_find("coll", "test")
		.unwrap()
		.unwrap_err()
		.unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.set_find("coll", "test2").unwrap().unwrap_err();
	assert!(commit.is_none());

	let commit = workspace.set_add_unchecked("coll", "test2").unwrap();
	assert_eq!(Record::message(&commit), "test2");

	let commit = workspace.set_find("coll", "test2").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test2");

	let commit = workspace.set_add_unchecked("coll", "test3").unwrap();
	assert_eq!(Record::message(&commit), "test3");
	let commit = workspace.set_add_unchecked("coll", "test4").unwrap();
	assert_eq!(Record::message(&commit), "test4");
	let commit = workspace.set_add_unchecked("coll", "test5").unwrap();
	assert_eq!(Record::message(&commit), "test5");

	let commit = workspace.set_find("coll", "test4").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test4");
	let commit = workspace.set_find("coll", "test5").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test5");
	let commit = workspace.set_find("coll", "test3").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test3");

	let commit = workspace.set_del_unchecked("coll", "test4").unwrap();
	assert_eq!(Record::message(&commit), "test4");
	let commit = workspace
		.set_find("coll", "test4")
		.unwrap()
		.unwrap_err()
		.unwrap();
	assert_eq!(Record::message(&commit), "test4");

	let commit = workspace.set_find("coll", "test3").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test3");
	let commit = workspace.set_find("coll", "test5").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test5");

	// now collect all of the operations we just did into a vector and make sure it's correct
	let records = workspace
		.walk_set("coll")
		.unwrap()
		.map(|r| r.unwrap())
		.map(|(record, op)| (Record::message(&record), op))
		.collect::<Vec<_>>();
	assert_eq!(records.len(), 7);
	assert_eq!(records[6], ("test".to_string(), SetOperation::Add));
	assert_eq!(records[5], ("test".to_string(), SetOperation::Del));
	assert_eq!(records[4], ("test2".to_string(), SetOperation::Add));
	assert_eq!(records[3], ("test3".to_string(), SetOperation::Add));
	assert_eq!(records[2], ("test4".to_string(), SetOperation::Add));
	assert_eq!(records[1], ("test5".to_string(), SetOperation::Add));
	assert_eq!(records[0], ("test4".to_string(), SetOperation::Del));
}

#[test]
fn test_set_get_all() {
	let workspace = create_test_workspace!();

	let commit = workspace.set_add_unchecked("coll", "test").unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.set_add_unchecked("coll", "test2").unwrap();
	assert_eq!(Record::message(&commit), "test2");

	let commit = workspace.set_add_unchecked("coll", "test3").unwrap();
	assert_eq!(Record::message(&commit), "test3");

	let commit = workspace.set_add_unchecked("coll", "test4").unwrap();
	assert_eq!(Record::message(&commit), "test4");

	let commit = workspace.set_add_unchecked("coll", "test5").unwrap();
	assert_eq!(Record::message(&commit), "test5");

	let commit = workspace.set_del_unchecked("coll", "test4").unwrap();
	assert_eq!(Record::message(&commit), "test4");

	let commit = workspace.set_del_unchecked("coll", "test3").unwrap();
	assert_eq!(Record::message(&commit), "test3");

	let commit = workspace.set_add_unchecked("coll", "test6").unwrap();
	assert_eq!(Record::message(&commit), "test6");

	let commit = workspace.set_add_unchecked("coll", "test3").unwrap();
	assert_eq!(Record::message(&commit), "test3");

	let records = workspace.set_get_all("coll").unwrap();
	assert_eq!(records.len(), 5);
	assert_eq!(records[0].message(), "test");
	assert_eq!(records[1].message(), "test2");
	assert_eq!(records[2].message(), "test5");
	assert_eq!(records[3].message(), "test6");
	assert_eq!(records[4].message(), "test3");
}

#[test]
fn test_set_checked() {
	let workspace = create_test_workspace!();

	let commit = workspace.set_add("coll", "test").unwrap().unwrap();
	assert_eq!(Record::message(&commit.0), "test");
	assert_eq!(commit.1, None);
	assert_eq!(workspace.walk_set("coll").unwrap().count(), 1);

	let commit = workspace.set_add("coll", "test").unwrap().unwrap_err();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(workspace.walk_set("coll").unwrap().count(), 1);

	let commit = workspace.set_add("coll", "test").unwrap().unwrap_err();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(workspace.walk_set("coll").unwrap().count(), 1);

	let commit = workspace.set_add_unchecked("coll", "test").unwrap();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(workspace.walk_set("coll").unwrap().count(), 2);

	let commit = workspace.set_add("coll", "test").unwrap().unwrap_err();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(workspace.walk_set("coll").unwrap().count(), 2);
}

#[test]
fn test_workspace() {
	let workspace = create_test_workspace!();

	assert_eq!(workspace.name().unwrap(), None);
	assert_eq!(workspace.description().unwrap(), None);
	workspace.set_name("test").unwrap();
	assert_eq!(workspace.name().unwrap().unwrap().message(), "test");
	workspace.set_description("test").unwrap();
	assert_eq!(workspace.description().unwrap().unwrap().message(), "test");
}

#[test]
fn test_project() {
	let workspace = create_test_workspace!();

	let project = workspace.create_project("test").unwrap().unwrap();
	assert_eq!(project.name().unwrap(), None);
	assert_eq!(project.description().unwrap(), None);

	let record = project.set_name("test").unwrap();
	assert_eq!(record.message(), "test");
	let record = project.set_description("test description").unwrap();
	assert_eq!(record.message(), "test description");
}

#[test]
fn test_ticket() {
	let workspace = create_test_workspace!();

	let project = workspace.create_project("test").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();
	assert_eq!(ticket.id(), 1);
	assert_eq!(ticket.slug(), "test-1");
	assert_eq!(ticket.title().unwrap(), None);

	ticket.set_title("test title").unwrap();
	assert_eq!(ticket.title().unwrap().unwrap().message(), "test title");
}
