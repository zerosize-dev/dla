//! SonarQube / SonarCloud. Sonar analyses on a server, so there is nothing to run
//! offline: without `sonar-scanner` the backend is `missing`; with it, `blocked`
//! because DLA does not fetch issues from a server.

use super::{Job, Outcome};

/// Reports Sonar as missing or blocked; it never produces hits here.
pub fn run(job: &Job) -> Outcome {
    if job.env.find_program("sonar-scanner").is_none() {
        return Outcome::missing(
            "sonar-scanner is not installed and Sonar rules need a SonarQube or SonarCloud server",
        );
    }
    Outcome::blocked(
        "Sonar rules (S5164, S2095) are evaluated by a server; DLA does not fetch server issues, so none were collected",
        vec![],
    )
}
