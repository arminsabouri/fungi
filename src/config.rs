use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub simulation: SimulationConfig,
    pub wallet_types: Vec<WalletTypeConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimulationConfig {
    pub seed: Option<u64>,
    pub max_timestep: u64,
    pub num_payment_obligations: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WalletTypeConfig {
    pub name: String,
    pub count: usize,
    pub strategies: Vec<String>,
    pub scorer: ScorerConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ScorerConfig {
    pub initiate_payjoin_utility_factor: f64,
    pub respond_to_payjoin_utility_factor: f64,
    pub payment_obligation_utility_factor: f64,
    pub multi_party_payjoin_utility_factor: f64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        // Validate strategy names
        let valid_strategies = [
            "UnilateralSpender",
            "BatchSpender",
            "PayjoinStrategy",
            "MultipartyPayjoinInitiatorStrategy",
            "MultipartyPayjoinParticipantStrategy",
        ];
        for wallet_type in &config.wallet_types {
            for strategy in &wallet_type.strategies {
                if !valid_strategies.contains(&strategy.as_str()) {
                    return Err(format!(
                        "Invalid strategy name: {}. Valid strategies are: {:?}",
                        strategy, valid_strategies
                    )
                    .into());
                }
            }
            if wallet_type.count == 0 {
                return Err(
                    format!("Wallet type '{}' must have count > 0", wallet_type.name).into(),
                );
            }
        }

        Ok(config)
    }

    pub fn total_wallets(&self) -> usize {
        self.wallet_types.iter().map(|wt| wt.count).sum()
    }
}
