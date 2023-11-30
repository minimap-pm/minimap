// This is NOT a complete rust file; it's a snippet that should
// be included into test modules for workspaces.
//
// Define your own `create_test_remote!()` macro that returns a
// `T: workspace` of some sort, and then include it:
//
// ```rust
// macro_rules! create_test_remote {
//     () => (MyRemote::open("some/default"));
//     ($suffix:expr) => (MyWorkspace(format!("some/default/{suffix}")));
// }
//
// include!("acceptance-tests.inc.rs");
// ```

use crate::*;

#[test]
fn test_commit() {
	let workspace = Workspace::open(create_test_remote!());

	let commit = workspace.remote().record_builder("coll").commit("test").unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.remote().get_record(&Record::id(&commit)).unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test");
}

#[test]
fn test_walk() {
	let workspace = Workspace::open(create_test_remote!());

	let first = workspace.remote().record_builder("coll").commit("test").unwrap();
	let second = workspace.remote().record_builder("coll").commit("test2").unwrap();

	let mut iter = workspace.remote().walk("coll").unwrap();
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
	let workspace = Workspace::open(create_test_remote!());

	workspace.remote().set_add_unchecked("coll", "test").unwrap();
	workspace.remote().set_del_unchecked("coll", "test").unwrap();
	workspace.remote().set_add_unchecked("coll", "test2").unwrap();
	workspace.remote().set_add_unchecked("coll", "test3").unwrap();
	workspace.remote().set_add_unchecked("coll", "test4").unwrap();
	workspace.remote().set_add_unchecked("coll", "test5").unwrap();
	workspace.remote().set_del_unchecked("coll", "test4").unwrap();

	let records = workspace
		.remote().walk_set("coll")
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
	let workspace = Workspace::open(create_test_remote!());

	let commit = workspace.remote().set_add_unchecked("coll", "test").unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.remote().set_find("coll", "test").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.remote().set_del_unchecked("coll", "test").unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace
		.remote().set_find("coll", "test")
		.unwrap()
		.unwrap_err()
		.unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.remote().set_find("coll", "test2").unwrap().unwrap_err();
	assert!(commit.is_none());

	let commit = workspace.remote().set_add_unchecked("coll", "test2").unwrap();
	assert_eq!(Record::message(&commit), "test2");

	let commit = workspace.remote().set_find("coll", "test2").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test2");

	let commit = workspace.remote().set_add_unchecked("coll", "test3").unwrap();
	assert_eq!(Record::message(&commit), "test3");
	let commit = workspace.remote().set_add_unchecked("coll", "test4").unwrap();
	assert_eq!(Record::message(&commit), "test4");
	let commit = workspace.remote().set_add_unchecked("coll", "test5").unwrap();
	assert_eq!(Record::message(&commit), "test5");

	let commit = workspace.remote().set_find("coll", "test4").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test4");
	let commit = workspace.remote().set_find("coll", "test5").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test5");
	let commit = workspace.remote().set_find("coll", "test3").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test3");

	let commit = workspace.remote().set_del_unchecked("coll", "test4").unwrap();
	assert_eq!(Record::message(&commit), "test4");
	let commit = workspace
		.remote().set_find("coll", "test4")
		.unwrap()
		.unwrap_err()
		.unwrap();
	assert_eq!(Record::message(&commit), "test4");

	let commit = workspace.remote().set_find("coll", "test3").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test3");
	let commit = workspace.remote().set_find("coll", "test5").unwrap().unwrap();
	assert_eq!(Record::message(&commit), "test5");

	// now collect all of the operations we just did into a vector and make sure it's correct
	let records = workspace
		.remote().walk_set("coll")
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
	let workspace = Workspace::open(create_test_remote!());

	let commit = workspace.remote().set_add_unchecked("coll", "test").unwrap();
	assert_eq!(Record::message(&commit), "test");

	let commit = workspace.remote().set_add_unchecked("coll", "test2").unwrap();
	assert_eq!(Record::message(&commit), "test2");

	let commit = workspace.remote().set_add_unchecked("coll", "test3").unwrap();
	assert_eq!(Record::message(&commit), "test3");

	let commit = workspace.remote().set_add_unchecked("coll", "test4").unwrap();
	assert_eq!(Record::message(&commit), "test4");

	let commit = workspace.remote().set_add_unchecked("coll", "test5").unwrap();
	assert_eq!(Record::message(&commit), "test5");

	let commit = workspace.remote().set_del_unchecked("coll", "test4").unwrap();
	assert_eq!(Record::message(&commit), "test4");

	let commit = workspace.remote().set_del_unchecked("coll", "test3").unwrap();
	assert_eq!(Record::message(&commit), "test3");

	let commit = workspace.remote().set_add_unchecked("coll", "test6").unwrap();
	assert_eq!(Record::message(&commit), "test6");

	let commit = workspace.remote().set_add_unchecked("coll", "test3").unwrap();
	assert_eq!(Record::message(&commit), "test3");

	let records = workspace.remote().set_get_all("coll").unwrap();
	assert_eq!(records.len(), 5);
	assert_eq!(records[0].message(), "test");
	assert_eq!(records[1].message(), "test2");
	assert_eq!(records[2].message(), "test5");
	assert_eq!(records[3].message(), "test6");
	assert_eq!(records[4].message(), "test3");

	let records = workspace.remote().set_get_all_reverse("coll").unwrap();
	assert_eq!(records.len(), 5);
	assert_eq!(records[0].message(), "test3");
	assert_eq!(records[1].message(), "test6");
	assert_eq!(records[2].message(), "test5");
	assert_eq!(records[3].message(), "test2");
	assert_eq!(records[4].message(), "test");
}

#[test]
fn test_set_checked() {
	let workspace = Workspace::open(create_test_remote!());

	let commit = workspace.remote().set_add("coll", "test").unwrap().unwrap();
	assert_eq!(Record::message(&commit.0), "test");
	assert_eq!(commit.1, None);
	assert_eq!(workspace.remote().walk_set("coll").unwrap().count(), 1);

	let commit = workspace.remote().set_add("coll", "test").unwrap().unwrap_err();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(workspace.remote().walk_set("coll").unwrap().count(), 1);

	let commit = workspace.remote().set_add("coll", "test").unwrap().unwrap_err();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(workspace.remote().walk_set("coll").unwrap().count(), 1);

	let commit = workspace.remote().set_add_unchecked("coll", "test").unwrap();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(workspace.remote().walk_set("coll").unwrap().count(), 2);

	let commit = workspace.remote().set_add("coll", "test").unwrap().unwrap_err();
	assert_eq!(Record::message(&commit), "test");
	assert_eq!(workspace.remote().walk_set("coll").unwrap().count(), 2);
}

#[test]
fn test_workspace() {
	let workspace = Workspace::open(create_test_remote!());

	assert_eq!(workspace.name().unwrap(), None);
	assert_eq!(workspace.description().unwrap(), None);
	workspace.set_name("test").unwrap();
	assert_eq!(workspace.name().unwrap().unwrap().message(), "test");
	workspace.set_description("test").unwrap();
	assert_eq!(workspace.description().unwrap().unwrap().message(), "test");
}

#[test]
fn test_project() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	assert_eq!(project.name().unwrap(), None);
	assert_eq!(project.description().unwrap(), None);

	assert!(workspace.create_project("test").unwrap().is_err());

	let record = project.set_name("test").unwrap();
	assert_eq!(record.message(), "test");
	let record = project.set_description("test description").unwrap();
	assert_eq!(record.message(), "test description");

	workspace.delete_project("test").unwrap().unwrap();
	workspace.delete_project("test").unwrap().unwrap_err();

	workspace.create_project("test").unwrap().unwrap();
	assert!(workspace.create_project("test").unwrap().is_err());
}

#[test]
fn test_ticket() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();
	assert_eq!(ticket.id(), 1);
	assert_eq!(ticket.slug(), "test-1");
	assert_eq!(ticket.title().unwrap(), None);

	ticket.set_title("test title").unwrap();
	assert_eq!(ticket.title().unwrap().unwrap().message(), "test title");

	let ticket2 = project.create_ticket().unwrap();
	assert_eq!(ticket2.id(), 2);
	assert_eq!(ticket2.slug(), "test-2");
	assert_eq!(ticket2.title().unwrap(), None);

	let ticket3 = project.create_ticket().unwrap();
	assert_eq!(ticket3.id(), 3);
	assert_eq!(ticket3.slug(), "test-3");
	assert_eq!(ticket3.title().unwrap(), None);

	ticket3.set_title("test title 3").unwrap();
	assert_eq!(ticket3.title().unwrap().unwrap().message(), "test title 3");

	// make sure that we didn't overwrite ticket 1 or something
	assert_eq!(ticket.title().unwrap().unwrap().message(), "test title");
}

#[test]
fn test_ticket_slug() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();
	assert_eq!(ticket.id(), 1);
	assert_eq!(ticket.slug(), "test-1");
	assert_eq!(ticket.title().unwrap(), None);

	ticket.set_title("test title").unwrap();
	assert_eq!(ticket.title().unwrap().unwrap().message(), "test title");

	// now try to fetch that ticket from the project
	let ticket2 = project.ticket(1).unwrap();
	assert_eq!(ticket2.id(), 1);
	assert_eq!(ticket2.slug(), "test-1");
	assert_eq!(ticket2.title().unwrap().unwrap().message(), "test title");

	// and now try to fetch it from the workspace
	let ticket3 = workspace.ticket("test-1").unwrap();
	assert_eq!(ticket3.id(), 1);
	assert_eq!(ticket3.slug(), "test-1");
	assert_eq!(ticket3.title().unwrap().unwrap().message(), "test title");
}

#[test]
fn test_ticket_comment() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();
	assert_eq!(ticket.id(), 1);
	assert_eq!(ticket.slug(), "test-1");
	assert_eq!(ticket.title().unwrap(), None);

	let comment = ticket.add_comment("test comment").unwrap();
	assert_eq!(comment.message(), "test comment");

	let comment2 = ticket.add_comment("test comment 2").unwrap();
	assert_eq!(comment2.message(), "test comment 2");

	// now iterate over the comments and make sure they're in the right order
	let comments = ticket.comments().unwrap().map(Result::unwrap).collect::<Vec<_>>();
	assert_eq!(comments.len(), 2);
	assert_eq!(comments[0].message(), "test comment 2");
	assert_eq!(comments[1].message(), "test comment");
}

#[test]
fn test_ticket_comment_attachment() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();
	assert_eq!(ticket.id(), 1);

	assert!(ticket.attachment("test").unwrap().is_none());

	ticket.upsert_attachment("test", b"test attachment").unwrap();

	let attachment = ticket.attachment("test").unwrap().unwrap();
	assert_eq!(attachment, b"test attachment");
}

#[test]
fn test_ticket_state() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();
	assert_eq!(ticket.id(), 1);
	assert_eq!(ticket.slug(), "test-1");
	assert_eq!(ticket.title().unwrap(), None);

	let (state, maybe_record) = ticket.state().unwrap();
	assert_eq!(state, TicketState::Open);
	assert!(maybe_record.is_none());
	assert!(ticket.is_open().unwrap());
	assert!(!ticket.is_closed().unwrap());

	let record = ticket.set_state(TicketState::Closed).unwrap();
	let (state, maybe_record) = ticket.state().unwrap();
	assert_eq!(state, TicketState::Closed);
	assert_eq!(maybe_record.unwrap().id(), record.id());
	assert!(!ticket.is_open().unwrap());
	assert!(ticket.is_closed().unwrap());

	let record = ticket.set_state(TicketState::Open).unwrap();
	let (state, maybe_record) = ticket.state().unwrap();
	assert_eq!(state, TicketState::Open);
	assert_eq!(maybe_record.unwrap().id(), record.id());
	assert!(ticket.is_open().unwrap());
	assert!(!ticket.is_closed().unwrap());

	// create a second ticket and make sure its state changes
	// don't affect the first ticket's.
	let ticket2 = project.create_ticket().unwrap();
	assert_eq!(ticket2.id(), 2);
	assert_eq!(ticket2.slug(), "test-2");
	assert!(ticket2.is_open().unwrap());

	ticket2.set_state(TicketState::Closed).unwrap();
	assert!(ticket2.is_closed().unwrap());
	assert!(ticket.is_open().unwrap());

	ticket2.set_state(TicketState::Open).unwrap();
	assert!(ticket2.is_open().unwrap());
	assert!(ticket.is_open().unwrap());

	ticket.set_state(TicketState::Closed).unwrap();
	assert!(ticket2.is_open().unwrap());
	assert!(ticket.is_closed().unwrap());
}

#[test]
fn test_ticket_dependency() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();

	let record1 = ticket.add_dependency("_", "foo-1").unwrap();
	let record2 = ticket.add_dependency("ext", "foo-1").unwrap();

	assert_ne!(record1.id(), record2.id());

	let record3 = ticket.add_dependency("_", "foo-1").unwrap();
	let record4 = ticket.add_dependency("_", "foo-2").unwrap();

	assert_eq!(record3.id(), record1.id());
	assert_ne!(record4.id(), record1.id());

	let record5 = ticket.add_dependency("ext", "foo-1").unwrap();
	let record6 = ticket.add_dependency("ext", "foo-2").unwrap();
	assert_eq!(record5.id(), record2.id());
	assert_ne!(record6.id(), record2.id());

	let record7 = ticket.add_dependency("invalid@foo", "baz").unwrap_err();
	match record7 {
		Error::MalformedOrigin(m) => assert_eq!(m, "invalid@foo".to_string()),
		_ => panic!("unexpected error: {:?}", record7),
	}

	let record8 = ticket.remove_dependency("_", "foo-2").unwrap();
	assert!(record8.is_some());

	let record9 = ticket.remove_dependency("_", "foo-none").unwrap();
	assert!(record9.is_none());

	let record10 = ticket.remove_dependency("ext", "foo-2").unwrap();
	assert!(record10.is_some());

	// now go through the dependencies and make sure all the ones we didn't remove
	// are still there.
	let deps = ticket.dependencies().unwrap();
	assert_eq!(deps.len(), 2);
	assert!(deps.contains(&("_".to_string(), "foo-1".to_string())));
	assert!(deps.contains(&("ext".to_string(), "foo-1".to_string())));
}

#[test]
fn test_self_dependencies() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();
	let ticket2 = project.create_ticket().unwrap();
	assert_eq!(ticket.id(), 1);
	assert_eq!(ticket2.id(), 2);

	ticket.add_dependency("_", "test-2").unwrap();

	let deps = ticket.dependencies().unwrap();
	assert_eq!(deps.len(), 1);
	assert!(deps.contains(&("_".to_string(), "test-2".to_string())));

	let deps = ticket2.dependencies().unwrap();
	assert_eq!(deps.len(), 0);

	let registry = DependencyRegistry::new();
	let mut found = false;
	for (origin, endpoint, status) in ticket.resolve_dependencies(&registry).unwrap().map(|r| r.unwrap()) {
		assert!(!found);
		found = true;
		assert_eq!(origin, "_");
		assert_eq!(endpoint, "test-2");
		assert_eq!(status, DependencyStatus::Pending);
	}
	assert!(found);

	ticket2.set_state(TicketState::Closed).unwrap();

	let mut found = false;
	for (origin, endpoint, status) in ticket.resolve_dependencies(&registry).unwrap().map(|r| r.unwrap()) {
		assert!(!found);
		found = true;
		assert_eq!(origin, "_");
		assert_eq!(endpoint, "test-2");
		assert_eq!(status, DependencyStatus::Complete);
	}
}

#[test]
fn test_subprojects() {
	let workspace = Workspace::open(create_test_remote!());

	let project = workspace.create_project("test").unwrap().unwrap();
	let subproject = project.create_project("sub").unwrap().unwrap();
	let ticket = project.create_ticket().unwrap();
	let subticket = subproject.create_ticket().unwrap();

	assert_eq!(ticket.id(), 1);
	assert_eq!(subticket.id(), 1);

	assert!(project.parent().unwrap().is_none());
	assert!(subproject.parent().unwrap().is_some());
	assert_eq!(subproject.parent().unwrap().unwrap().slug(), "test");

	assert_eq!(subproject.slug(), "sub");

	assert_eq!(workspace.ticket("sub-1").unwrap().slug(), "sub-1");
}
