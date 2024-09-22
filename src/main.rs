use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyDirectory, ReplyEntry, ReplyOpen,
    Request,
};
use libc::ENOENT;
use std::ffi::{OsStr, OsString};
use std::time::{Duration, UNIX_EPOCH};

const TTL: Duration = Duration::from_secs(1);

const ROOT_DIRECTORY: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

struct Entry {
    name: OsString,
    children: Vec<u64>,
    attr: FileAttr,
}

struct HelloFS {
    entries: Vec<Option<Entry>>,
}

impl Filesystem for HelloFS {
    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        reply: ReplyEntry,
    ) {
        let name = name.to_str().unwrap();
        println!("mkdir parent {parent:?} name {name:?}");
        let ino = (self.entries.len() + 1) as u64;
        self.entries.push(Some(Entry {
            name: OsString::from(name),
            children: Vec::new(),
            attr: FileAttr {
                ino: ino as u64,
                size: 0,
                blocks: 0,
                atime: UNIX_EPOCH, // 1970-01-01 00:00:00
                mtime: UNIX_EPOCH,
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: FileType::Directory,
                perm: 0o755,
                nlink: 2,
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0,
                blksize: 512,
            },
        }));
        self.entries[(parent - 1) as usize]
            .as_mut()
            .unwrap()
            .children
            .push(ino);
        reply.entry(
            &TTL,
            &self.entries[self.entries.len() - 1].as_ref().unwrap().attr,
            0,
        );
    }

    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup: parent:{parent:?} name:{name:?}");
        for ino in &self.entries[(parent - 1) as usize]
            .as_ref()
            .unwrap()
            .children
        {
            let child = self.entries[(ino - 1) as usize].as_ref().unwrap();
            if name == child.name {
                reply.entry(&TTL, &child.attr, 0);
                return;
            }
        }

        reply.error(ENOENT);
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir: ino {ino:?}");
        if ino > self.entries.len() as u64 {
            reply.error(ENOENT);
            return;
        }

        let entry = self.entries[(ino - 1) as usize].as_ref().unwrap();
        for (i, ino) in entry.children.iter().enumerate().skip(offset as usize) {
            let child = &self.entries[(ino - 1) as usize].as_ref().unwrap();
            if reply.add(
                child.attr.ino,
                (i + 1) as i64,
                child.attr.kind,
                child.name.clone(),
            ) {
                break;
            }
        }
        reply.ok();
    }

    fn opendir(&mut self, _req: &Request<'_>, ino: u64, _flags: i32, reply: ReplyOpen) {
        println!("opendir: ino {ino:?}");
        reply.opened(0, 0);
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        println!("getattr: ino {ino:?}");
        if ino > self.entries.len() as u64 {
            reply.error(ENOENT);
            return;
        }
        reply.attr(&TTL, &self.entries[ino as usize - 1].as_ref().unwrap().attr);
    }
}

fn main() {
    let mut hello = HelloFS {
        entries: Vec::new(),
    };

    hello.entries.push(Some(Entry {
        name: OsString::from("root"),
        children: Vec::new(),
        attr: FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH, // 1970-01-01 00:00:00
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        },
    }));

    let mountpoint = OsString::from("/tmp/rust-fat");
    fuser::mount2(hello, mountpoint, &[MountOption::AutoUnmount]).unwrap();
}
