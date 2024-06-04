use mock::*;

mod mock;

#[test]
fn test_voting_rewards() {
    new_test_ext().execute_with(|| {
        zero_min_burn();

        let treasury_address = DaoTreasuryAddress::<Test>::get();
        add_balance(treasury_address, to_nano(30_000));

        const AUTHOR: u32 = 0;
        const SUBNET_ID: u16 = 0;
        const MODULE_ID: u32 = 0;

        register(AUTHOR, SUBNET_ID, MODULE_ID, to_nano(3));
        config(1, 100);

        let origin = get_origin(AUTHOR);
        assert_ok!(Governance::do_add_subnet_custom_proposal(
            origin.clone(),
            SUBNET_ID,
            b"first proposal".to_vec()
        ));
        assert_ok!(Governance::do_add_subnet_custom_proposal(
            origin,
            SUBNET_ID,
            b"second proposal".to_vec()
        ));

        const FIRST_VOTER: u32 = 1;
        stake(FIRST_VOTER, SUBNET_ID, MODULE_ID, to_nano(4));
        vote(FIRST_VOTER, 0, true);

        const SECOND_VOTER: u32 = 2;
        stake(SECOND_VOTER, SUBNET_ID, MODULE_ID, to_nano(4));
        vote(SECOND_VOTER, 0, true);
        vote(SECOND_VOTER, 1, true);

        dbg!(get_balance(FIRST_VOTER));
        dbg!(get_balance(SECOND_VOTER));

        step_block(100);

        step_block(
            SubnetGovernanceConfig::<Test>::get(SUBNET_ID).proposal_reward_interval as usize,
        );

        dbg!(get_balance(FIRST_VOTER));
        dbg!(get_balance(SECOND_VOTER));

        assert!(get_balance(FIRST_VOTER) < get_balance(SECOND_VOTER));
    });
}
