use crate::pipeline::block::NuDataSensorSettings;

use super::ExecutionContext;

impl ExecutionContext {
    pub(super) async fn execute_nudata_sensor(
        &mut self,
        settings: &NuDataSensorSettings,
    ) -> crate::error::Result<()> {
        let site = self.variables.interpolate(&settings.site);
        let init_url = self.variables.interpolate(&settings.init_url);
        let href = self.variables.interpolate(&settings.href);
        let proxy = self.variables.interpolate(&settings.proxy);
        let solver_url = self.variables.interpolate(&settings.solver_url);

        let body = if !site.is_empty() {
            serde_json::json!({
                "site": site,
                "proxy": if proxy.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(proxy) },
                "href": if href.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(href) },
            })
        } else {
            serde_json::json!({
                "initUrl": init_url,
                "mode": settings.mode,
                "sdkVersion": settings.sdk_version,
                "proxy": if proxy.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(proxy) },
                "href": if href.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(href) },
            })
        };

        let endpoint = if !site.is_empty() {
            format!("{}/nudata/solve", solver_url)
        } else {
            format!("{}/nudata/solve-custom", solver_url)
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                crate::error::AppError::Pipeline(format!("NuData HTTP client error: {e}"))
            })?;

        let resp = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body).unwrap_or_default())
            .send()
            .await
            .map_err(|e| {
                crate::error::AppError::Pipeline(format!(
                    "NuData solver unreachable: {e} — is nudata-solver running on {solver_url}?"
                ))
            })?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(crate::error::AppError::Pipeline(format!(
                "NuData solver returned {status}: {text}"
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
            crate::error::AppError::Pipeline(format!("NuData solver bad JSON: {e}"))
        })?;

        let nds_pmd = json["nds-pmd"].as_str().ok_or_else(|| {
            crate::error::AppError::Pipeline("NuData solver returned no nds-pmd field".into())
        })?;

        self.variables
            .set_user(&settings.output_var, nds_pmd.to_string(), settings.capture);

        if !settings.sid_var.is_empty() {
            if let Some(sid) = json["sid"].as_str() {
                self.variables
                    .set_user(&settings.sid_var, sid.to_string(), settings.capture);
            }
        }

        Ok(())
    }
}
