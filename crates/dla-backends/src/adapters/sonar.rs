//! SonarQube / SonarCloud. Sonar analyses on a server, and DLA does not query one, so
//! there is nothing to run and nothing an install could change: the backend is always
//! `blocked` with a fixed reason that says so and names the offline backends that ran for
//! the same shapes. It never carries an install hint.

use super::{Job, Outcome};

/// The offline backends with a rule for what Sonar's S5164 (`ThreadLocal` not removed) and
/// S2095 (resource not closed) look for. Others that ran (detekt, the DLA rules) have no
/// rule for those shapes, so naming them would claim coverage that does not exist.
const SAME_SHAPES: [&str; 3] = ["errorprone", "spotbugs", "codeql"];

/// The row's reason. `ran` names the backends that produced a result in this sweep (only
/// those with a rule for the same shapes are listed); `host_set` is true when
/// `SONAR_HOST_URL` is set.
pub fn reason(ran: &[String], host_set: bool) -> String {
    let ran: Vec<&String> = ran
        .iter()
        .filter(|b| SAME_SHAPES.contains(&b.as_str()))
        .collect();
    let ran: Vec<&str> = ran.iter().map(|b| b.as_str()).collect();
    let offline = if ran.is_empty() {
        "no offline backend ran for the same shapes".to_owned()
    } else {
        format!(
            "offline backends that ran for the same shapes: {}",
            ran.join(", ")
        )
    };
    let server = if host_set {
        "DLA does not fetch server issues even when SONAR_HOST_URL is set"
    } else {
        "which DLA does not query"
    };
    format!(
        "server-only: S5164 and S2095 are evaluated on a SonarQube or SonarCloud server, {server}; {offline}"
    )
}

/// Reports Sonar as blocked; it never produces hits here.
pub fn run(job: &Job) -> Outcome {
    let host_set = std::env::var_os("SONAR_HOST_URL").is_some_and(|v| !v.is_empty());
    Outcome::blocked(reason(&job.ran, host_set), vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reason_is_fixed_server_only_and_names_the_backends_that_ran() {
        let r = reason(&["errorprone".into(), "spotbugs".into()], false);
        assert_eq!(
            r,
            "server-only: S5164 and S2095 are evaluated on a SonarQube or SonarCloud server, which DLA does not query; offline backends that ran for the same shapes: errorprone, spotbugs"
        );
        assert!(reason(&[], false).ends_with("no offline backend ran for the same shapes"));
        let r = reason(&["detekt".into(), "dla".into()], true);
        assert!(r.contains("even when SONAR_HOST_URL is set"), "{r}");
        // detekt has no rule for ThreadLocal removal or unclosed resources.
        assert!(
            r.ends_with("no offline backend ran for the same shapes"),
            "{r}"
        );
        let r = reason(&["detekt".into(), "codeql".into()], false);
        assert!(r.ends_with("same shapes: codeql"), "{r}");
        for r in [reason(&[], false), reason(&["a".into()], true)] {
            assert!(!r.contains("install"), "no install hint in {r}");
        }
    }
}
