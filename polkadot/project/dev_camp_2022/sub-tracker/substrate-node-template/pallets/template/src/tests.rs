use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok,bounded_vec};

#[test]
fn it_creates_transit_node() {
	new_test_ext().execute_with(|| {
		//Check number of transit nodes. should be 0
		assert_eq!(AssetTracking::count_for_transit_point(),0);
		// Create Transit Node
		assert_ok!(AssetTracking::create_new_transit_node(Origin::root(),1,bounded_vec![]));
		// Check number of transit nodes. should be 1
		assert_eq!(AssetTracking::count_for_transit_point(),1);
		// Try to create the same transit node again. Should fail
	    assert_noop!(AssetTracking::create_new_transit_node(Origin::root(),1,bounded_vec![]),
		Error::<Test>::TransitPointAlreadyExists);
	});
}

#[test]
fn it_removes_transit_node() {
	new_test_ext().execute_with(|| {
		// Check number of transit nodes. should be 0
		assert_eq!(AssetTracking::count_for_transit_point(),0);
		// Create Transit Node 1
		assert_ok!(AssetTracking::create_new_transit_node(Origin::root(),1,bounded_vec![]));
		// Check number of transit nodes. should be 1
		assert_eq!(AssetTracking::count_for_transit_point(),1);
		// Remove Transit Node 1
		assert_ok!(AssetTracking::remove_transit_node(Origin::root(),1));
		// Check number of transit nodes. should be 0
		assert_eq!(AssetTracking::count_for_transit_point(),0);
		// Try to remove node 2. Should fail
		assert_noop!(AssetTracking::remove_transit_node(Origin::root(),2),
		Error::<Test>::TransitPointNotFound);
	});
}



