mod helpers;
use cosmwasm_std::Uint128;
use helpers::{TestEnv, TokenfactoryIssuer};
use osmosis_testing::{
    cosmrs::proto::cosmos::bank::v1beta1::QueryBalanceRequest, Account, RunnerError,
};
use tokenfactory_issuer::{msg::AllowanceInfo, ContractError};

#[test]
fn set_burner_performed_by_contract_owner_should_work() {
    let env = TestEnv::default();
    let owner = &env.test_accs[0];
    let non_owner = &env.test_accs[1];

    let allowance = 1000000;
    env.tokenfactory_issuer
        .set_burner(&non_owner.address(), allowance, owner)
        .unwrap();

    let burn_allowance = env
        .tokenfactory_issuer
        .query_burn_allowance(&env.test_accs[1].address())
        .unwrap()
        .allowance;

    assert_eq!(burn_allowance.u128(), allowance);
}

#[test]
fn set_burner_performed_by_non_contract_owner_should_fail() {
    let env = TestEnv::default();
    let non_owner = &env.test_accs[1];

    let allowance = 1000000;

    let err = env
        .tokenfactory_issuer
        .set_burner(&non_owner.address(), allowance, non_owner)
        .unwrap_err();

    assert_eq!(
        err,
        TokenfactoryIssuer::execute_error(ContractError::Unauthorized {})
    );
}

#[test]
fn set_allowance_to_0_should_remove_it_from_storage() {
    let env = TestEnv::default();
    let owner = &env.test_accs[0];
    let burner = &env.test_accs[1];

    // set allowance to some value
    let allowance = 1000000;
    env.tokenfactory_issuer
        .set_burner(&burner.address(), allowance, owner)
        .unwrap();

    // set allowance to 0
    env.tokenfactory_issuer
        .set_burner(&burner.address(), 0, owner)
        .unwrap();

    // check if key for the minter address is removed
    assert_eq!(
        env.tokenfactory_issuer
            .query_burn_allowances(None, None)
            .unwrap()
            .allowances,
        vec![]
    );
}

#[test]
fn used_up_allowance_should_be_removed_from_storage() {
    let env = TestEnv::default();
    let owner = &env.test_accs[0];
    let burner = &env.test_accs[1];

    // set allowance to some value
    let allowance = 1000000;
    env.tokenfactory_issuer
        .set_minter(&burner.address(), allowance, owner)
        .unwrap();

    // mint the whole allowance to be burned the same amount later
    env.tokenfactory_issuer
        .mint(&burner.address(), allowance, burner)
        .unwrap();

    env.tokenfactory_issuer
        .set_burner(&burner.address(), allowance, owner)
        .unwrap();

    // use all allowance
    env.tokenfactory_issuer
        .burn(&burner.address(), allowance, burner)
        .unwrap();

    // check if key for the burner address is removed
    assert_eq!(
        env.tokenfactory_issuer
            .query_burn_allowances(None, None)
            .unwrap()
            .allowances,
        vec![]
    );
}

#[test]
fn burn_whole_balance_but_less_than_or_eq_allowance_should_work_and_deduct_allowance() {
    let cases = vec![
        (u128::MAX, u128::MAX),
        (u128::MAX, u128::MAX - 1),
        (u128::MAX, 1),
        (2, 1),
        (1, 1),
    ];

    cases.into_iter().for_each(|(allowance, burn_amount)| {
        let env = TestEnv::default();
        let owner = &env.test_accs[0];
        let denom = env.tokenfactory_issuer.query_denom().unwrap().denom;

        let burner = &env.test_accs[1];
        let burn_to = &env.test_accs[2];

        // mint
        env.tokenfactory_issuer
            .set_minter(&burner.address(), allowance, owner)
            .unwrap();

        env.tokenfactory_issuer
            .mint(&burn_to.address(), burn_amount, burner)
            .unwrap();

        // burn
        env.tokenfactory_issuer
            .set_burner(&burner.address(), allowance, owner)
            .unwrap();

        env.tokenfactory_issuer
            .burn(&burn_to.address(), burn_amount, burner)
            .unwrap();

        // check if allowance is deducted properly
        let resulted_allowance = env
            .tokenfactory_issuer
            .query_burn_allowance(&burner.address())
            .unwrap()
            .allowance
            .u128();

        assert_eq!(resulted_allowance, allowance - burn_amount);

        // check if resulted balance is 0
        let amount = env
            .bank()
            .query_balance(&QueryBalanceRequest {
                address: burn_to.address(),
                denom,
            })
            .unwrap()
            .balance
            .unwrap()
            .amount;

        assert_eq!(amount, "0");
    });
}

#[test]
fn burn_more_than_balance_should_fail_and_not_deduct_allowance() {
    let cases = vec![(u128::MAX - 1, u128::MAX), (1, 2)];

    cases.into_iter().for_each(|(balance, burn_amount)| {
        let env = TestEnv::default();
        let owner = &env.test_accs[0];
        let denom = env.tokenfactory_issuer.query_denom().unwrap().denom;

        let burner = &env.test_accs[1];
        let burn_from = &env.test_accs[2];

        let allowance = burn_amount;

        // mint
        env.tokenfactory_issuer
            .set_minter(&burner.address(), balance, owner)
            .unwrap();

        env.tokenfactory_issuer
            .mint(&burn_from.address(), balance, burner)
            .unwrap();

        // burn
        env.tokenfactory_issuer
            .set_burner(&burner.address(), allowance, owner)
            .unwrap();

        let err = env
            .tokenfactory_issuer
            .burn(&burn_from.address(), allowance, burner)
            .unwrap_err();

        assert_eq!(
            err,
            RunnerError::ExecuteError {
                msg: format!("failed to execute message; message index: 0: dispatch: submessages: {balance}{denom} is smaller than {burn_amount}{denom}: insufficient funds")
            }
        );

        // check if allowance stays the same
        let resulted_allowance = env
           .tokenfactory_issuer
           .query_burn_allowance(&burner.address())
           .unwrap()
           .allowance
           .u128();

       assert_eq!(resulted_allowance, allowance);
    });
}

#[test]
fn burn_over_allowance_should_fail_and_not_deduct_allowance() {
    let cases = vec![(u128::MAX - 1, u128::MAX), (0, 1)];

    cases.into_iter().for_each(|(allowance, burn_amount)| {
        let env = TestEnv::default();
        let owner = &env.test_accs[0];

        let burner = &env.test_accs[1];
        let burn_from = &env.test_accs[2];

        env.tokenfactory_issuer
            .set_burner(&burner.address(), allowance, owner)
            .unwrap();

        let err = env
            .tokenfactory_issuer
            .burn(&burn_from.address(), burn_amount, burner)
            .unwrap_err();

        assert_eq!(
            err,
            TokenfactoryIssuer::execute_error(ContractError::not_enough_burn_allowance(
                burn_amount,
                allowance
            ))
        );

        // check if allowance stays the same
        let resulted_allowance = env
            .tokenfactory_issuer
            .query_burn_allowance(&burner.address())
            .unwrap()
            .allowance
            .u128();

        assert_eq!(resulted_allowance, allowance);
    });
}

#[test]
fn burn_0_should_fail_and_not_deduct_allowance() {
    let cases = vec![(u128::MAX, 0), (0, 0)];

    cases.into_iter().for_each(|(allowance, burn_amount)| {
        let env = TestEnv::default();
        let owner = &env.test_accs[0];

        let burner = &env.test_accs[1];
        let burn_to = &env.test_accs[2];

        env.tokenfactory_issuer
            .set_burner(&burner.address(), allowance, owner)
            .unwrap();

        let err = env
            .tokenfactory_issuer
            .burn(&burn_to.address(), burn_amount, burner)
            .unwrap_err();

        assert_eq!(
            err,
            TokenfactoryIssuer::execute_error(ContractError::ZeroAmount {})
        );

        // check if allowance stays the same
        let resulted_allowance = env
            .tokenfactory_issuer
            .query_burn_allowance(&burner.address())
            .unwrap()
            .allowance
            .u128();

        assert_eq!(resulted_allowance, allowance);
    });
}

#[test]
fn test_query_burn_allowances_within_default_limit() {
    helpers::test_query_within_default_limit::<AllowanceInfo, _, _>(
        |(i, addr)| AllowanceInfo {
            address: addr.to_string(),
            allowance: Uint128::from((i as u128 + 1) * 10000u128), // generate distincted allowance
        },
        |env| {
            move |allowance| {
                let owner = &env.test_accs[0];
                env.tokenfactory_issuer
                    .set_burner(&allowance.address, allowance.allowance.u128(), owner)
                    .unwrap();
            }
        },
        |env| {
            move |start_after, limit| {
                env.tokenfactory_issuer
                    .query_burn_allowances(start_after, limit)
                    .unwrap()
                    .allowances
            }
        },
    );
}

#[test]
fn test_query_burn_allowance_over_default_limit() {
    helpers::test_query_over_default_limit::<AllowanceInfo, _, _>(
        |(i, addr)| AllowanceInfo {
            address: addr.to_string(),
            allowance: Uint128::from((i as u128 + 1) * 10000u128), // generate distincted allowance
        },
        |env| {
            move |allowance| {
                let owner = &env.test_accs[0];
                env.tokenfactory_issuer
                    .set_burner(&allowance.address, allowance.allowance.u128(), owner)
                    .unwrap();
            }
        },
        |env| {
            move |start_after, limit| {
                env.tokenfactory_issuer
                    .query_burn_allowances(start_after, limit)
                    .unwrap()
                    .allowances
            }
        },
    );
}
