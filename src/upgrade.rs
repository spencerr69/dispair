use serde::{Deserialize, Serialize, ser::Error};

pub struct PlayerState {
    upgrades: Vec<UpgradeNode>,
    inventory: Inventory,
}

#[derive(Deserialize, Serialize)]
pub struct UpgradeNode {
    title: String,
    description: String,
    children: Option<Vec<UpgradeNode>>,
    id: u32,
    value: f64,
}

pub struct Inventory {
    gold: i32,
}

pub fn get_upgrade_tree() -> Result<Vec<UpgradeNode>, serde_json::Error> {
    let get_file = std::fs::read_to_string("upgrades.json")
        .map_err(|_| serde_json::Error::custom("naurrr"))?;
    let upgrade_tree: Vec<UpgradeNode> = serde_json::from_str(get_file.as_str())?;
    Ok(upgrade_tree)
}
