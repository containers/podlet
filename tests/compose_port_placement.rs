#![allow(unused_crate_dependencies)]

#[cfg(test)]
mod tests {
    use std::process::{Command, Output};

    const COMPOSE_FILE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/compose-port-placement.yaml"
    );

    fn run(args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_podlet"))
            .arg("compose")
            .args(args)
            .arg(COMPOSE_FILE)
            .output()
            .expect("podlet should run")
    }

    fn file<'a>(output: &'a str, name: &str) -> &'a str {
        let header = format!("# FileName={name}\n");
        output
            .split(&header)
            .nth(1)
            .expect("named file should be present in output")
            .split("\n---\n")
            .next()
            .expect("file should have content")
    }

    #[test]
    fn pod_ports_default_and_explicit() {
        let default = run(&["--pod"]);
        assert!(
            default.status.success(),
            "{}",
            String::from_utf8_lossy(&default.stderr)
        );
        let explicit = run(&["--pod", "--port-placement=pod"]);
        assert!(
            explicit.status.success(),
            "{}",
            String::from_utf8_lossy(&explicit.stderr)
        );
        assert_eq!(default.stdout, explicit.stdout);

        let output = String::from_utf8(default.stdout).expect("output should be UTF-8");
        let pod = file(&output, "port-placement");
        assert!(pod.contains("PublishPort=127.0.0.1:8080:80"));
        assert!(pod.contains("PublishPort=8443:443"));
        assert!(pod.contains("PublishPort=3478:3478/udp"));
        assert!(!file(&output, "port-placement-web").contains("PublishPort="));
        assert!(!file(&output, "port-placement-server").contains("PublishPort="));
    }

    #[test]
    fn container_ports_default_and_explicit() {
        let default = run(&[]);
        assert!(
            default.status.success(),
            "{}",
            String::from_utf8_lossy(&default.stderr)
        );
        let explicit = run(&["--port-placement=container"]);
        assert!(
            explicit.status.success(),
            "{}",
            String::from_utf8_lossy(&explicit.stderr)
        );
        assert_eq!(default.stdout, explicit.stdout);

        let output = String::from_utf8(default.stdout).expect("output should be UTF-8");
        let web = file(&output, "web");
        assert!(web.contains("PublishPort=127.0.0.1:8080:80"));
        assert!(web.contains("PublishPort=8443:443"));
        let server = file(&output, "server");
        assert!(server.contains("PublishPort=3478:3478/udp"));
        assert!(!output.contains("[Pod]"));
    }

    #[test]
    fn unsupported_port_placement_combinations_fail() {
        let container_in_pod = run(&["--pod", "--port-placement=container"]);
        assert!(!container_in_pod.status.success());
        assert!(String::from_utf8_lossy(&container_in_pod.stderr).contains("Omit `--pod`"));

        let pod_without_pod = run(&["--port-placement=pod"]);
        assert!(!pod_without_pod.status.success());
        assert!(String::from_utf8_lossy(&pod_without_pod.stderr).contains("requires `--pod`"));
    }
}
