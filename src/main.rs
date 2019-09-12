use termiku::pty::pty;

fn main() {
    pty("ping", &["8.8.8.8"]);
}
