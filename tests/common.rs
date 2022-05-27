fn run(executable: &str, args: Vec<&str>) {
    let mut cmd = Command::new(executable);
    cmd.arg(args.into_inte().join(" "));
    let output = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("failed to execute process");

    let code = output.status.code().expect("should have status code");
    stdout().write_all(&output.stdout).unwrap();
    if code != 0 {
        stderr().write_all(&output.stderr).unwrap();
        panic!("failed to run wasi-cloud");
    }
}
