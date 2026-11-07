//! A small reader for SARIF 2.1.0 logs: the fields every adapter needs.

use serde_json::Value;

/// One SARIF result, reduced to what becomes a hit.
#[derive(Debug, Clone, PartialEq)]
pub struct SarifResult {
    /// `ruleId`.
    pub rule: String,
    /// `message.text`, first line.
    pub message: String,
    /// `physicalLocation.artifactLocation.uri` of the first location.
    pub uri: String,
    /// `region.startLine` (1 when absent).
    pub line: u32,
    /// `logicalLocations[0].fullyQualifiedName`, when present.
    pub logical: Option<String>,
}

/// The results of every run in `log`. Results without a rule or a location are
/// skipped.
pub fn results(log: &Value) -> Vec<SarifResult> {
    let mut out = Vec::new();
    let Some(runs) = log.get("runs").and_then(Value::as_array) else {
        return out;
    };
    for run in runs {
        let Some(rs) = run.get("results").and_then(Value::as_array) else {
            continue;
        };
        for r in rs {
            let rule = r
                .get("ruleId")
                .and_then(Value::as_str)
                .or_else(|| r.pointer("/rule/id").and_then(Value::as_str));
            let loc = r.pointer("/locations/0");
            let uri = loc
                .and_then(|l| l.pointer("/physicalLocation/artifactLocation/uri"))
                .and_then(Value::as_str);
            let (Some(rule), Some(uri)) = (rule, uri) else {
                continue;
            };
            let line = loc
                .and_then(|l| l.pointer("/physicalLocation/region/startLine"))
                .and_then(Value::as_u64)
                .map_or(1, |n| u32::try_from(n).unwrap_or(1))
                .max(1);
            let message = r
                .pointer("/message/text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_owned();
            let logical = loc
                .and_then(|l| l.pointer("/logicalLocations/0/fullyQualifiedName"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            out.push(SarifResult {
                rule: rule.to_owned(),
                message,
                uri: uri.to_owned(),
                line,
                logical,
            });
        }
    }
    out
}

/// True when any run reports `executionSuccessful: false`.
pub fn execution_failed(log: &Value) -> bool {
    log.get("runs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| r.get("invocations").and_then(Value::as_array))
        .flatten()
        .any(|i| i.get("executionSuccessful").and_then(Value::as_bool) == Some(false))
}

/// The first JSON value in `text`, tolerating trailing non-JSON output.
pub fn first_json(text: &str) -> Option<Value> {
    let t = text.trim_start();
    let start = t.find(['{', '['])?;
    let mut de = serde_json::Deserializer::from_str(&t[start..]).into_iter::<Value>();
    de.next()?.ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_rule_site_and_message() {
        let log = json!({"runs":[{"results":[
            {"ruleId":"B019","message":{"text":"first\nsecond"},
             "locations":[{"physicalLocation":{"artifactLocation":{"uri":"a.py"},"region":{"startLine":14}},
                           "logicalLocations":[{"fullyQualifiedName":"A.f()"}]}]},
            {"message":{"text":"no rule"},"locations":[]},
            {"ruleId":"X","message":{"text":"no region"},
             "locations":[{"physicalLocation":{"artifactLocation":{"uri":"b.py"}}}]}
        ]}]});
        let r = results(&log);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].rule, "B019");
        assert_eq!(r[0].line, 14);
        assert_eq!(r[0].message, "first");
        assert_eq!(r[0].logical.as_deref(), Some("A.f()"));
        assert_eq!(r[1].line, 1);
    }

    #[test]
    fn execution_failure_and_prefix_noise() {
        let log = json!({"runs":[{"invocations":[{"executionSuccessful":false}],"results":[]}]});
        assert!(execution_failed(&log));
        let v = first_json("{\"a\":1}\n8 issues:\n* x: 1").unwrap();
        assert_eq!(v["a"], 1);
        assert!(first_json("no json here").is_none());
    }
}
