mod cli;
mod workflows;

fn toma_command() -> std::process::Command {
    let executable = std::env::var_os("NEXTEST_BIN_EXE_toma_cli").unwrap_or_else(|| env!("CARGO_BIN_EXE_toma-cli").into());
    std::process::Command::new(executable)
}

#[test]
fn cli_executable_is_resolved() {
    assert!(std::path::Path::new(toma_command().get_program()).is_file());
}
