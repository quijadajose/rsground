pub mod error;

use std::path::{Path, PathBuf};
use std::{fs, io};

use hakoniwa::{Command, Container, Output};

pub struct Runner {
    container: Container,
    temp_home: PathBuf,
}

impl Runner {
    fn create_container(temp_home: &PathBuf) -> Container {
        fs::create_dir(temp_home).expect("Cannot create home");

        Container::new()
            .hostname("rsground")
            .rootfs(concat!(env!("CARGO_MANIFEST_DIR"), "/lxc_rootfs"))
            .tmpfsmount("/tmp")
            .devfsmount("/dev")
            .procfsmount("/proc")
            .uidmap(1001)
            .gidmap(100)
            .bindmount_rw(temp_home.to_str().unwrap(), "/home")
            // FIXME: This needs to set resource limit
            // .setrlimit(hakoniwa::Rlimit::*, soft_limit, hard_limit)
            .clone()
    }

    pub fn new() -> Result<Self, ()> {
        let mut temp_home = PathBuf::from("/tmp");
        temp_home.push(uuid::Uuid::new_v4().simple().to_string());

        let container = Self::create_container(&temp_home);
        Ok(Self {
            container,
            temp_home,
        })
    }

    pub fn create_file(
        &self,
        container_path: impl AsRef<str>,
        content: impl AsRef<str>,
    ) -> io::Result<()> {
        let mut home = self.temp_home.clone();
        home.push(container_path.as_ref());

        fs::write(home, content.as_ref())
    }

    pub fn spawn(
        &self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<hakoniwa::Child, hakoniwa::Error> {
        self.container
            .command(cmd.as_ref())
            .args(args)
            .env("HOME", "/home")
            .env("PATH", "/bin")
            .env("LD_LIBRARY_PATH", "/lib:/lib64:/libexec")
            .stdin(hakoniwa::Stdio::Inherit)
            .stdout(hakoniwa::Stdio::Inherit)
            .stderr(hakoniwa::Stdio::Inherit)
            .current_dir("/home")
            .spawn()
    }

    pub fn run(
        &self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Output, hakoniwa::Error> {
        self.container
            .command(cmd.as_ref())
            .args(args)

            // .command("/bin/sh")
            // .arg("-c")
            // .arg(&format!(
            //     "{} {}",
            //     cmd.as_ref(),
            //     args.into_iter()
            //         .map(|a| a.as_ref().to_owned())
            //         .collect::<Vec<_>>()
            //         .join(" ")
            // ))
            .env("HOME", "/home")
            .env("PATH", "/bin")
            .env("PROG", "/bin/cc")
            .env("LD_LIBRARY_PATH", "/lib:/lib64:/libexec")
            .current_dir("/home")
            // .spawn()
            .output()
            .map(|mut output| {
                if output.stderr.starts_with(b"hakoniwa: ") {
                    output.stderr = output.stderr[10..].to_vec()
                }

                output
            })
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.temp_home).unwrap();
    }
}
