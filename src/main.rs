use termiku::bridge::spawn_process;

fn main() {
    spawn_process("ping", &["8.8.8.8"]);
}
