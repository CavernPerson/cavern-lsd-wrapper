use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cosmwasm_std::Decimal;
use cosmwasm_std::Uint128;
use cw20::Expiration;
use cw20::Logo;

#[cw_serde]
pub enum ExecuteMsg {
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    /// Burn is a base message to destroy tokens forever
    Burn {
        amount: Uint128,
    },
    BurnAll {},
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Only with "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Only with "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Only with "approval" extension. Destroys tokens forever
    BurnFrom {
        owner: String,
        amount: Uint128,
    },
    /// Only with the "mintable" extension. If the contract can transfer enough lsd funds from the caller, creates amount new tokens
    /// and adds to the recipient balance.
    Mint {
        recipient: String,
        amount: Uint128,
    },
    /// Same as the Mint function but you specify the amount of funds you want to send to the contract instead
    MintWith {
        recipient: String,
        lsd_amount: Uint128,
    },
    /// Only with the "mintable" extension. The current minter may set
    /// a new minter. Setting the minter to None will remove the
    /// token's minter forever.
    UpdateMinter {
        new_minter: Option<String>,
    },
    /// Only with the "marketing" extension. If authorized, updates marketing metadata.
    /// Setting None/null for any of these will leave it unchanged.
    /// Setting Some("") will clear this field on the contract storage
    UpdateMarketing {
        /// A URL pointing to the project behind this token.
        project: Option<String>,
        /// A longer description of the token and it's utility. Designed for tooltips or such
        description: Option<String>,
        /// The address (if any) who can update this data structure
        marketing: Option<String>,
    },
    /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
    UploadLogo(Logo),

    /// Wrapper specific message
    Decompound {
        recipient: Option<String>,
    },
}

#[derive(Default)]
#[cw_serde]
pub struct AccruedRewards {
    pub luna_rewards: Uint128,
    pub lsd_rewards: Uint128,
}

#[cw_serde]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub exchange_rate: Decimal,
}
