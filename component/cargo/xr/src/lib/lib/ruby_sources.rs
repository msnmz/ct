use super::{
    err,
    fs,
    te,
    Error,
    Path,
    Result,
};
#[cfg(not(feature = "git2"))]
use __devcmd as __inline;
#[cfg(feature = "git2")]
use __git2_airbrake as __inline;
pub use __inline::*;

#[cfg(feature = "git2")]
pub fn find_rubies<C, PR>(
    repo: &git2::Repository,
    tree: &git2::Tree,
    mut c: C,
    pr: PR,
) -> Result<()>
where
    C: FnMut(&str, &[u8]) -> Result<()>,
    PR: FnOnce(Error),
{
    let mode = git2::TreeWalkMode::PostOrder;
    let mut find_error = None;
    tree.walk(mode, |_, entry| {
        if let Some(rb) = entry.name().filter(|n| n.ends_with(".rb")) {
            let r = (|| -> Result<()> {
                let obj = te!(entry.to_object(repo));
                match obj.into_blob() {
                    Ok(blob) => {
                        c(rb, blob.content())?;
                    }
                    other => log::debug!("NOT A BLOB: {:?}", other),
                }
                Ok(())
            })();

            if let Err(e) = r {
                find_error = Some(e);
                return git2::TreeWalkResult::Abort;
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    if let Some(err) = find_error {
        pr(err)
    }

    Ok(())
}

pub mod gh {
    pub mod airbrake {
        pub const ID: &str = "airbrake";
    }
}

#[cfg(feature = "git2")]
pub mod __git2_airbrake {
    use super::*;
    pub mod github {
        use super::*;

        pub const PREFIX: &str = "_.popgit-slaesh-";

        pub struct Github<S>(pub S);

        impl<S: std::fmt::Display> Github<S> {
            fn url(&self) -> String {
                format!("https://github.com/{}", self.full_id())
            }
            fn id(&self) -> String {
                format!("{}", self.0)
            }
            fn full_id(&self) -> String {
                let id = self.id();
                if id.contains('/') {
                    id
                } else {
                    format!("{id}/{id}", id = id)
                }
            }
            fn target(&self) -> String {
                format!("{}{}", PREFIX, self.full_id())
            }
            pub fn git_clone(&self) -> Result<git2::Repository> {
                let mut repo = git2::build::RepoBuilder::new();
                let url = &self.url();
                let target = self.target();
                let md = fs::metadata(&target);
                let repo = match md {
                    Ok(md) if md.is_file() => {
                        panic!("File exists: {:?}", target);
                    }
                    Ok(md) if md.is_dir() => {
                        te!(git2::Repository::open(target))
                    }
                    _ => {
                        eprintln!("* Cloning into {:?} <- {}", target, url);
                        te!(repo.bare(true).clone(url, Path::new(&target)))
                    }
                };
                eprintln!("* Repo open");

                Ok(repo)
            }
        }
    }

    pub fn find_rubies_in_repo<C>(id: &str, mut callback: C) -> Result<()>
    where
        C: FnMut(&str, &[u8]) -> Result<()>,
    {
        let repo = te!(github::Github(id).git_clone());

        let head = te!(repo.head());
        let head = te!(head.resolve());
        let head = te!(head.peel_to_commit());
        let head = te!(head.tree());

        let mut find_error = None;
        te!(find_rubies(
            &repo,
            &head,
            |id, cont| { callback(id, cont) },
            |error| find_error = Some(error)
        ));
        if let Some(err) = find_error {
            err!(err)
        } else {
        }

        Ok(())
    }
}

#[cfg(not(feature = "git2"))]
pub mod __devcmd {
    use super::*;

    pub fn find_rubies_in_repo<C>(_: &str, mut callback: C) -> Result<()>
    where
        C: FnMut(&str, &[u8]) -> Result<()>,
    {
        let path = "./ruby-parse-test.rb";
        let mut file = te!(fs::File::open(path), path);
        let mut string = String::new();
        let _ = te!(std::io::Read::read_to_string(&mut file, &mut string));
        te!(callback(path, string.as_bytes()));
        Ok(())
    }
}
