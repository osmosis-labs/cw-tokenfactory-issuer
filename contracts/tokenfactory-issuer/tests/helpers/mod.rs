// the code is used in test but reported as dead code
// see https://github.com/rust-lang/rust/issues/46379
#![allow(dead_code)]

use cosmwasm_std::Coin;

use osmosis_std::types::osmosis::tokenfactory::v1beta1::QueryDenomAuthorityMetadataRequest;
use osmosis_testing::{
    cosmrs::proto::{
        cosmos::bank::v1beta1::{MsgSend, MsgSendResponse},
        cosmwasm::wasm::v1::{
            MsgExecuteContractResponse, MsgMigrateContract, MsgMigrateContractResponse,
        },
    },
    Account, Bank, Module, OsmosisTestApp, Runner, RunnerError, RunnerExecuteResult,
    SigningAccount, TokenFactory, Wasm,
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::path::PathBuf;
use std::rc::Rc;
use tokenfactory_issuer::msg::{
    BlacklisteesResponse, BlacklisterAllowancesResponse, Metadata, MigrateMsg,
};
use tokenfactory_issuer::{
    msg::{
        AllowanceResponse, AllowancesResponse, DenomResponse, ExecuteMsg,
        FreezerAllowancesResponse, InstantiateMsg, IsFrozenResponse, OwnerResponse, QueryMsg,
        StatusResponse,
    },
    ContractError,
};

pub struct TestEnv {
    pub test_accs: Vec<SigningAccount>,
    pub tokenfactory_issuer: TokenfactoryIssuer,
}

impl TestEnv {
    pub fn new(instantiate_msg: InstantiateMsg, signer_index: usize) -> Result<Self, RunnerError> {
        let app = OsmosisTestApp::new();
        let test_accs_count: u64 = 4;
        let test_accs = Self::create_default_test_accs(&app, test_accs_count);

        let tokenfactory_issuer =
            TokenfactoryIssuer::new(app, &instantiate_msg, &test_accs[signer_index])?;

        Ok(Self {
            test_accs,
            tokenfactory_issuer,
        })
    }

    pub fn create_default_test_accs(
        app: &OsmosisTestApp,
        test_accs_count: u64,
    ) -> Vec<SigningAccount> {
        let default_initial_balance = [Coin::new(100_000_000_000, "uosmo")];

        app.init_accounts(&default_initial_balance, test_accs_count)
            .unwrap()
    }

    pub fn app(&self) -> &OsmosisTestApp {
        &self.tokenfactory_issuer.app
    }
    pub fn tokenfactory(&self) -> TokenFactory<'_, OsmosisTestApp> {
        TokenFactory::new(self.app())
    }

    pub fn bank(&self) -> Bank<'_, OsmosisTestApp> {
        Bank::new(self.app())
    }

    pub fn token_admin(&self, denom: &str) -> String {
        self.tokenfactory()
            .query_denom_authority_metadata(&QueryDenomAuthorityMetadataRequest {
                denom: denom.to_string(),
            })
            .unwrap()
            .authority_metadata
            .unwrap()
            .admin
    }

    pub fn send_tokens(
        &self,
        to: String,
        coins: Vec<Coin>,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgSendResponse> {
        self.app().execute::<MsgSend, MsgSendResponse>(
            MsgSend {
                from_address: signer.address(),
                to_address: to,
                amount: coins
                    .into_iter()
                    .map(
                        |c| osmosis_testing::cosmrs::proto::cosmos::base::v1beta1::Coin {
                            denom: c.denom,
                            amount: c.amount.to_string(),
                        },
                    )
                    .collect(),
            },
            "/cosmos.bank.v1beta1.MsgSend",
            signer,
        )
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new(
            InstantiateMsg::NewToken {
                subdenom: "uusd".to_string(),
            },
            0,
        )
        .unwrap()
    }
}

#[derive(Debug)]
pub struct TokenfactoryIssuer {
    pub app: OsmosisTestApp,
    pub code_id: u64,
    pub contract_addr: String,
}

impl TokenfactoryIssuer {
    pub fn new(
        app: OsmosisTestApp,
        instantiate_msg: &InstantiateMsg,
        signer: &SigningAccount,
    ) -> Result<Self, RunnerError> {
        let wasm = Wasm::new(&app);
        let token_creation_fee = Coin::new(10000000, "uosmo");

        let code_id = wasm
            .store_code(&Self::get_wasm_byte_code(), None, signer)?
            .data
            .code_id;
        let contract_addr = wasm
            .instantiate(
                code_id,
                &instantiate_msg,
                Some(&signer.address()),
                None,
                &[token_creation_fee],
                signer,
            )?
            .data
            .address;

        Ok(Self {
            app,
            code_id,
            contract_addr,
        })
    }

    // executes
    pub fn execute(
        &self,
        execute_msg: &ExecuteMsg,
        funds: &[Coin],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        let wasm = Wasm::new(&self.app);
        wasm.execute(&self.contract_addr, execute_msg, funds, signer)
    }

    pub fn change_contract_owner(
        &self,
        new_owner: &str,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::ChangeContractOwner {
                new_owner: new_owner.to_string(),
            },
            &[],
            signer,
        )
    }
    pub fn change_tokenfactory_admin(
        &self,
        new_admin: &str,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::ChangeTokenFactoryAdmin {
                new_admin: new_admin.to_string(),
            },
            &[],
            signer,
        )
    }
    pub fn set_denom_metadata(
        &self,
        metadata: Metadata,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(&ExecuteMsg::SetDenomMetadata { metadata }, &[], signer)
    }

    pub fn set_minter(
        &self,
        address: &str,
        allowance: u128,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::SetMinter {
                address: address.to_string(),
                allowance: allowance.into(),
            },
            &[],
            signer,
        )
    }
    pub fn mint(
        &self,
        address: &str,
        amount: u128,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::Mint {
                to_address: address.to_string(),
                amount: amount.into(),
            },
            &[],
            signer,
        )
    }

    pub fn set_burner(
        &self,
        address: &str,
        allowance: u128,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::SetBurner {
                address: address.to_string(),
                allowance: allowance.into(),
            },
            &[],
            signer,
        )
    }
    pub fn burn(
        &self,
        address: &str,
        amount: u128,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::Burn {
                from_address: address.to_string(),
                amount: amount.into(),
            },
            &[],
            signer,
        )
    }

    pub fn set_freezer(
        &self,
        address: &str,
        status: bool,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::SetFreezer {
                address: address.to_string(),
                status,
            },
            &[],
            signer,
        )
    }

    pub fn set_blacklister(
        &self,
        address: &str,
        status: bool,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::SetBlacklister {
                address: address.to_string(),
                status,
            },
            &[],
            signer,
        )
    }

    pub fn freeze(
        &self,
        status: bool,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(&ExecuteMsg::Freeze { status }, &[], signer)
    }

    pub fn blacklist(
        &self,
        address: &str,
        status: bool,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse> {
        self.execute(
            &ExecuteMsg::Blacklist {
                address: address.to_string(),
                status,
            },
            &[],
            signer,
        )
    }

    // queries
    pub fn query<T>(&self, query_msg: &QueryMsg) -> Result<T, RunnerError>
    where
        T: DeserializeOwned,
    {
        let wasm = Wasm::new(&self.app);
        wasm.query(&self.contract_addr, query_msg)
    }

    pub fn query_denom(&self) -> Result<DenomResponse, RunnerError> {
        self.query(&QueryMsg::Denom {})
    }

    pub fn query_is_freezer(&self, address: &str) -> Result<StatusResponse, RunnerError> {
        self.query(&QueryMsg::IsFreezer {
            address: address.to_string(),
        })
    }

    pub fn query_is_blacklister(&self, address: &str) -> Result<StatusResponse, RunnerError> {
        self.query(&QueryMsg::IsBlacklister {
            address: address.to_string(),
        })
    }

    pub fn query_freezer_allowances(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<FreezerAllowancesResponse, RunnerError> {
        self.query(&QueryMsg::FreezerAllowances { start_after, limit })
    }

    pub fn query_blacklister_allowances(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<BlacklisterAllowancesResponse, RunnerError> {
        self.query(&QueryMsg::BlacklisterAllowances { start_after, limit })
    }

    pub fn query_blacklistees(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<BlacklisteesResponse, RunnerError> {
        self.query(&QueryMsg::Blacklistees { start_after, limit })
    }

    pub fn query_is_frozen(&self) -> Result<IsFrozenResponse, RunnerError> {
        self.query(&QueryMsg::IsFrozen {})
    }

    pub fn query_is_blacklisted(&self, address: &str) -> Result<StatusResponse, RunnerError> {
        self.query(&QueryMsg::IsBlacklisted {
            address: address.to_string(),
        })
    }
    pub fn query_owner(&self) -> Result<OwnerResponse, RunnerError> {
        self.query(&QueryMsg::Owner {})
    }
    pub fn query_mint_allowance(&self, address: &str) -> Result<AllowanceResponse, RunnerError> {
        self.query(&QueryMsg::MintAllowance {
            address: address.to_string(),
        })
    }

    pub fn query_mint_allowances(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<AllowancesResponse, RunnerError> {
        self.query(&QueryMsg::MintAllowances { start_after, limit })
    }

    pub fn query_burn_allowance(&self, address: &str) -> Result<AllowanceResponse, RunnerError> {
        self.query(&QueryMsg::BurnAllowance {
            address: address.to_string(),
        })
    }

    pub fn query_burn_allowances(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<AllowancesResponse, RunnerError> {
        self.query(&QueryMsg::BurnAllowances { start_after, limit })
    }

    pub fn migrate(
        &self,
        testdata: &str,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgMigrateContractResponse> {
        let wasm = Wasm::new(&self.app);
        let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let wasm_byte_code =
            std::fs::read(manifest_path.join("tests").join("testdata").join(testdata)).unwrap();

        let code_id = wasm.store_code(&wasm_byte_code, None, signer)?.data.code_id;
        self.app.execute(
            MsgMigrateContract {
                sender: signer.address(),
                contract: self.contract_addr.clone(),
                code_id,
                msg: serde_json::to_vec(&MigrateMsg {}).unwrap(),
            },
            "/cosmwasm.wasm.v1.MsgMigrateContract",
            signer,
        )
    }

    fn get_wasm_byte_code() -> Vec<u8> {
        let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        std::fs::read(
            manifest_path
                .join("..")
                .join("..")
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
                .join("tokenfactory_issuer.wasm"),
        )
        .unwrap()
    }

    pub fn execute_error(err: ContractError) -> RunnerError {
        RunnerError::ExecuteError {
            msg: format!(
                "failed to execute message; message index: 0: {}: execute wasm contract failed",
                err
            ),
        }
    }
}

pub fn test_query_within_default_limit<QueryResult, SetStateClosure, QueryStateClosure>(
    gen_result: impl FnMut((usize, &String)) -> QueryResult,
    set_state: impl Fn(Rc<TestEnv>) -> SetStateClosure,
    query_state: impl Fn(Rc<TestEnv>) -> QueryStateClosure,
) where
    QueryResult: PartialEq + Debug + Clone,
    SetStateClosure: Fn(QueryResult),
    QueryStateClosure: Fn(Option<String>, Option<u32>) -> Vec<QueryResult>,
{
    let env = Rc::new(TestEnv::default());
    let test_accs_count = 10;
    let test_accs_with_allowance =
        TestEnv::create_default_test_accs(&env.tokenfactory_issuer.app, test_accs_count);

    let mut sorted_addrs = test_accs_with_allowance
        .iter()
        .map(|acc| acc.address())
        .collect::<Vec<_>>();
    sorted_addrs.sort();

    let allowances = sorted_addrs
        .iter()
        .enumerate()
        .map(gen_result)
        .collect::<Vec<_>>();

    allowances
        .iter()
        .for_each(|allowance| set_state(env.clone())(allowance.clone()));

    let query = query_state(env);

    // let <n> be allowance for the sorted_addrs with index n

    // query from start with default limit
    // = [<0>..<10>] (since test_accs_count is 10)
    assert_eq!(query(None, None), allowances);

    // query from start with limit 1
    // = [<0>]
    assert_eq!(query(None, Some(1)), allowances[0..1]);

    // query start after sorted_addrs[1], limit 1
    // = [<2>]
    assert_eq!(
        query(Some(sorted_addrs[1].clone()), Some(1)),
        allowances[2..3]
    );

    // query start after sorted_addrs[1], limit 10
    // = [<2>..<10>] (since test_accs_count is 10)
    assert_eq!(
        query(Some(sorted_addrs[1].clone()), Some(10)),
        allowances[2..10]
    );

    // query start after sorted_addrs[9], with default limit
    // = []
    assert_eq!(query(Some(sorted_addrs[9].clone()), None), vec![]);
}

pub fn test_query_over_default_limit<QueryResult, SetStateClosure, QueryStateClosure>(
    gen_result: impl FnMut((usize, &String)) -> QueryResult,
    set_state: impl Fn(Rc<TestEnv>) -> SetStateClosure,
    query_state: impl Fn(Rc<TestEnv>) -> QueryStateClosure,
) where
    QueryResult: PartialEq + Debug + Clone,
    SetStateClosure: Fn(QueryResult),
    QueryStateClosure: Fn(Option<String>, Option<u32>) -> Vec<QueryResult>,
{
    let env = Rc::new(TestEnv::default());
    let test_accs_count = 40;
    let test_accs_with_allowance =
        TestEnv::create_default_test_accs(&env.tokenfactory_issuer.app, test_accs_count);

    let mut sorted_addrs = test_accs_with_allowance
        .iter()
        .map(|acc| acc.address())
        .collect::<Vec<_>>();
    sorted_addrs.sort();

    let allowances = sorted_addrs
        .iter()
        .enumerate()
        .map(gen_result)
        .collect::<Vec<_>>();

    allowances
        .iter()
        .for_each(|allowance| set_state(env.clone())(allowance.clone()));

    let query = query_state(env);

    // let <n> be allowance for the sorted_addrs with index n

    // query from start with default limit
    // = [<0>..<10>]
    assert_eq!(query(None, None), allowances[..10]);

    // query start after sorted_addrs[4] with default limit
    // = [<5>..<15>] (<5> is after <4>, <15> is <5> + limit 10)
    assert_eq!(
        query(Some(sorted_addrs[4].clone()), None),
        allowances[5..15]
    );

    // max limit = 30
    assert_eq!(query(None, Some(40)), allowances[..30]);

    // start after nth, get n+1 .. n+1+limit (30)
    assert_eq!(
        query(Some(sorted_addrs[4].clone()), Some(40)),
        allowances[5..35]
    );
}
