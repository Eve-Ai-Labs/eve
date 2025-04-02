pub fn get_whitelist_form() -> Option<&'static str> {
    option_env!("WHITELIST_FORM")
}

pub fn get_api_orchestrator() -> &'static str {
    option_env!("ORCHESTRATOR_RPC").unwrap_or("http://127.0.0.1:1733")
}

pub fn url_suffix() -> String {
    let now = option_env!("NOW").unwrap_or("000");
    format!("?b={}", now)
}
