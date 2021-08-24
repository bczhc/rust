use git2::{Commit, Repository, Revwalk};
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();

    let repo_path = "/home/bczhc/github/rust";

    let repository = Repository::open(repo_path).unwrap();
    let mut walker = repository.revwalk().unwrap();

    walker.push_head().unwrap();
    for oid in walker {
        let oid = oid.unwrap();
        let commit = repository.find_commit(oid).unwrap();
        let author = commit.author();
        map.insert(
            String::from(author.email().unwrap()),
            String::from(author.name().unwrap()),
        );
    }

    for (email, name) in &map {
        for (email2, name2) in &map {
            if name == name2 && email != email2 {
                println!("{:?}", (name, email, email2));
            }
        }
    }
}
