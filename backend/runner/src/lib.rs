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

    pub fn copy_file_from_runner(
        &self,
        other: &Runner,
        host_path: impl AsRef<Path>,
        other_path: impl AsRef<Path>,
    ) {
        let mut host_file_path = self.temp_home.clone();
        host_file_path.push(host_path);

        let mut other_file_path = other.temp_home.clone();
        other_file_path.push(other_path);

        std::fs::copy(other_file_path, host_file_path).expect("Skill issuer de manual");
    }

    fn run_command(cmd: &mut Command) -> Result<Output, hakoniwa::Error> {
        cmd.env("HOME", "/home")
            .env("PATH", "/bin")
            .env("PROG", "/bin/cc")
            .env("LD_LIBRARY_PATH", "/lib:/lib64:/libexec")
            .current_dir("/home")
            .output()
            .map(|mut output| {
                if output.stderr.starts_with(b"hakoniwa: ") {
                    output.stderr = output.stderr[10..].to_vec()
                }

                output
            })
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
        Self::run_command(self.container.command(cmd.as_ref()).args(args))
    }

    pub fn run_bash(
        &self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Output, hakoniwa::Error> {
        Self::run_command(self.container.command("/bin/bash").arg("-c").arg(&format!(
                "{} {}",
                cmd.as_ref(),
                args.into_iter()
                    .map(|a| a.as_ref().to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            )))
    }

    pub fn run_rustc(
        &self,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Output, hakoniwa::Error> {
        Self::run_command(
            self.container
                .command("/bin/rustc")
                // -C linker=/bin/ld -C link-args=-L/lib -C link-args=-L/lib/gcc/x86_64-unknown-linux-gnu/14.2.1
                .args(args),
        )
    }

    pub fn patch_binary(&self, path: impl AsRef<str>) -> Result<Output, hakoniwa::Error> {
        Self::run_command(
            self.container
                .command("/bin/patchelf")
                .arg("--set-interpreter")
                .arg("/lib/ld-linux-x86-64.so.2")
                .arg(path.as_ref()),
        )
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.temp_home).unwrap();
    }
}
