use reqwest::{Client, RequestBuilder, StatusCode};

use super::protocol::{
    EvidencePayload, EventPayload, OperatorAction, PendingRun, RegisterRequest, RegisterResponse,
    RunResult, Step, StepCompleteRequest,
};
use super::TransportResult;

#[derive(Debug, Clone)]
pub struct HttpTransport {
    client: Client,
    base_url: String,
    agent_id: String,
    auth_token: Option<String>,
}

impl HttpTransport {
    pub fn new(base_url: impl Into<String>, agent_id: impl Into<String>) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        if base_url.starts_with("http://") && !base_url.contains("127.0.0.1") && !base_url.contains("localhost") {
            eprintln!("[WARN] transport: connecting over plaintext HTTP to {}. Use HTTPS in production.", base_url);
        }
        Self {
            client: Client::new(),
            base_url,
            agent_id: agent_id.into(),
            auth_token: None,
        }
    }

    /// Set a pre-shared auth token. All subsequent requests will include
    /// `Authorization: Bearer <token>` header.
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Attach auth header to a request builder if a token is configured.
    fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.auth_token {
            Some(token) => req.bearer_auth(token),
            None => req,
        }
    }

    pub async fn register(
        &self,
        hostname: &str,
        ip: &str,
        os: &str,
        arch: &str,
        user: &str,
    ) -> TransportResult<String> {
        let body = RegisterRequest {
            id: if self.agent_id.is_empty() {
                None
            } else {
                Some(self.agent_id.clone())
            },
            hostname: hostname.to_string(),
            ip: ip.to_string(),
            os: os.to_string(),
            arch: arch.to_string(),
            user: user.to_string(),
        };

        let req = self
            .client
            .post(format!("{}/api/agents/register", self.base_url))
            .json(&body);

        let response: RegisterResponse = self
            .auth(req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response.id)
    }

    pub async fn heartbeat(&self) -> TransportResult<()> {
        let req = self.client.post(format!(
            "{}/api/agents/{}/heartbeat",
            self.base_url, self.agent_id
        ));

        self.auth(req).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn poll_pending(&self) -> TransportResult<Vec<PendingRun>> {
        let req = self.client.get(format!(
            "{}/api/runs/pending/{}",
            self.base_url, self.agent_id
        ));

        let response = self.auth(req).send().await?;

        if response.status() == StatusCode::FORBIDDEN {
            return Ok(Vec::new());
        }

        let runs = response.error_for_status()?.json::<Vec<PendingRun>>().await?;
        Ok(runs)
    }

    pub async fn post_result(&self, run_id: &str, result: RunResult) -> TransportResult<()> {
        let req = self
            .client
            .post(format!("{}/api/runs/{}/result", self.base_url, run_id))
            .json(&result);

        self.auth(req).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn post_event(&self, run_id: &str, level: &str, message: &str) -> TransportResult<()> {
        let body = EventPayload {
            run_id: Some(run_id.to_string()),
            agent_id: Some(self.agent_id.clone()),
            level: level.to_string(),
            message: message.to_string(),
        };

        let req = self
            .client
            .post(format!("{}/api/events", self.base_url))
            .json(&body);

        self.auth(req).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_steps(&self, run_id: &str) -> TransportResult<Vec<Step>> {
        let req = self
            .client
            .get(format!("{}/api/runs/{}/steps", self.base_url, run_id));

        let steps = self
            .auth(req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(steps)
    }

    pub async fn get_operator_actions(&self, run_id: &str) -> TransportResult<Vec<OperatorAction>> {
        let req = self.client.get(format!(
            "{}/api/runs/{}/operator-actions",
            self.base_url, run_id
        ));

        let actions = self
            .auth(req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(actions)
    }

    pub async fn post_evidence(&self, evidence: EvidencePayload) -> TransportResult<()> {
        let req = self
            .client
            .post(format!("{}/api/evidence", self.base_url))
            .json(&evidence);

        self.auth(req).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn complete_step(
        &self,
        run_id: &str,
        step_id: &str,
        success: bool,
        result_json: Option<String>,
    ) -> TransportResult<()> {
        let body = StepCompleteRequest {
            success,
            result_json,
        };

        let req = self
            .client
            .post(format!(
                "{}/api/runs/{}/steps/{}/complete",
                self.base_url, run_id, step_id
            ))
            .json(&body);

        self.auth(req).send().await?.error_for_status()?;
        Ok(())
    }
}
