use crate::error::ResponseError::InvalidProfileParameters;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) address: String,
    pub(crate) port: u16,
}

pub(crate) fn resolve_config_path(args: &[String]) -> anyhow::Result<String> {
    if args.len() != 2 {
        return Ok("./config/default".to_string());
    }

    const P: &str = "-p";
    const PROFILE: &str = "--profile";

    match args {
        [flag, profile] if matches!(flag.as_str(), P | PROFILE) => {
            Ok(format!("./config/{profile}"))
        }
        _ => Err(InvalidProfileParameters(P, PROFILE).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_config_path;
    use quickcheck::quickcheck;

    quickcheck! {
        #[test]
        fn resolves_arbitrary_profile_for_supported_flags(
            profile: String,
            use_long_flag: bool
        ) -> bool {
            let flag = if use_long_flag { "--profile" } else { "-p" };
            let args = [flag.to_string(), profile.clone()];
            matches!(
                resolve_config_path(&args),
                Ok(path) if path == format!("./config/{profile}")
            )
        }
    }

    #[test]
    fn resolves_default_profile_for_empty_arguments() {
        assert_eq!(resolve_config_path(&[]).unwrap(), "./config/default");
    }

    #[test]
    fn resolves_default_profile_for_invalid_flag() {
        let args = ["--invalid".to_string(), "test".to_string()];
        let result = resolve_config_path(&args);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid parameters. Use -p or --profile to specify a profile."
        );
    }
}
