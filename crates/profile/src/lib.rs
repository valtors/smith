use smith_config::SmithConfig;

pub fn switch(config: &mut SmithConfig, profile: &str) -> Result<String, String> {
    config.set_profile(profile);
    Ok(format!("switched to profile: {}", profile))
}

pub fn list(config: &SmithConfig) -> Vec<String> {
    config.list_profiles()
}

pub fn current(config: &SmithConfig) -> &str {
    &config.active_profile
}

pub fn assign(config: &mut SmithConfig, server_name: &str, profile: &str) -> Result<(), String> {
    let server = config.servers.iter_mut()
        .find(|s| s.name == server_name)
        .ok_or(format!("server not found: {}", server_name))?;
    server.profile = profile.to_string();
    Ok(())
}
