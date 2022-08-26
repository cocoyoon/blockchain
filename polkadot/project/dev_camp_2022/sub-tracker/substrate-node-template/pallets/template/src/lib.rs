#![cfg_attr(not(feature = "std"), no_std)]

  pub use pallet::*;

  #[cfg(test)]
  mod mock;

  #[cfg(test)]
  mod tests;

  #[frame_support::pallet]
  pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Randomness},
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::{
		TypeInfo,
	};
	use sp_io::hashing::blake2_128;
	use sp_runtime::ArithmeticError;
	use sp_std::{vec,vec::Vec};


	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Shipment<T: Config> {
		pub creator: T::AccountId,
		pub fees: Option<BalanceOf<T>>,
		pub owner_index: u8,
		pub route: BoundedVec<T::AccountId,T::MaxSize>,
		pub destination: T::AccountId,
		pub uid: u64,
		pub status: ShipmentStatus,
	}

	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum ShipmentStatus {
		InTransit,
		Delivered,
		Failed,
	}

	// The struct on which we build all of our Pallet logic.
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

    /* Placeholder for defining custom types. */

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type KeyRandomNess: Randomness<Self::Hash, Self::BlockNumber>;
		type MaxSize: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TransitPointCreated(T::AccountId),
		TransitPointRemoved(T::AccountId),
		NeighbourUpdated(T::AccountId,T::AccountId),
		ShipmentCreated(T::AccountId),
		ShipmentUpdated(T::AccountId),
		ShipmentReceived(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidUID,
		InvalidShipmentUID,
		InvalidRoute,
		InvalidKey,
		KeyNotFound,
		ShipmentAlreadyExists,
		ShipmentKeyAlreadyExists,
		ShipmentNotFound,
		TransitPointAlreadyExists,
		TransitNodesOverFlow,
		TransitPointNotFound,
		UIDNotFound,
		UnauthorizedCaller,
		CallerIsNotFirstNode
	}

	#[pallet::storage]
	#[pallet::getter(fn count_for_transit_point)]
	pub(super) type CountForTransitPoints<T:Config> = StorageValue<
		_,
		u64,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn route_costs)]
	pub(super) type RouteCosts<T:Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		u32,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn route_vec)]
	pub(super) type RouteVector<T:Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		Vec<T::AccountId>,
		OptionQuery,
	>;

	// shipment_uid -> key map
	#[pallet::storage]
	#[pallet::getter(fn shipment_uid_to_key)]
	pub(super) type UIDToKey<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		[u8; 16],
		OptionQuery,
	>;

	// shipment_uid -> shipment map
	#[pallet::storage]
	#[pallet::getter(fn uid_to_shipment)]
	pub(super) type UIDToShipment<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		Shipment<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn shipment_uid)]
	pub(super) type ShipmentUID<T:Config> = StorageValue<
		_,
		u64,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn transit_nodes)]
	pub(super) type TransitNodes<T:Config> = StorageValue<
		_,
		Vec<T::AccountId>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type Nonce<T:Config> = StorageValue<
		_,
		u32,
		ValueQuery,
	>;


    #[pallet::call]
    impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn create_new_transit_node(
			origin: OriginFor<T>,
			transit_node: T::AccountId,
			neighbours: BoundedVec<(T::AccountId, u32), T::MaxSize>
		) -> DispatchResult {

			ensure_root(origin)?;
			ensure!(!Self::transit_nodes().contains(&transit_node), Error::<T>::TransitPointAlreadyExists);
			ensure!(
				neighbours.iter().all(|neighbour| neighbour.0 != transit_node && Self::transit_nodes().contains(&neighbour.0)),
				Error::<T>::InvalidRoute);

			for neighbour in neighbours.iter() {
				RouteCosts::<T>::insert(transit_node.clone(), neighbour.0.clone(), neighbour.1);
				RouteCosts::<T>::insert(neighbour.0.clone(), transit_node.clone(), neighbour.1);
			}

			TransitNodes::<T>::append(transit_node.clone());
			let transit_point_counts = Self::count_for_transit_point().checked_add(1).ok_or(ArithmeticError::Overflow)?;
			CountForTransitPoints::<T>::put(transit_point_counts);

			Self::deposit_event(Event::TransitPointCreated(transit_node));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn update_neighbour(
			origin: OriginFor<T>,
			node1: T::AccountId,
			node2: T::AccountId,
			cost: u32
		) ->DispatchResult {
			ensure_root(origin)?;
			ensure!(Self::transit_nodes().contains(&node1) && Self::transit_nodes().contains(&node2), Error::<T>::TransitPointNotFound);

			RouteCosts::<T>::insert(node1.clone(),node2.clone(),cost.clone());
			RouteCosts::<T>::insert(node2.clone(),node1.clone(),cost.clone());

			Self::deposit_event(Event::NeighbourUpdated(node1,node2));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_transit_node(origin: OriginFor<T>, transit_node: T::AccountId) -> DispatchResult {

			ensure_root(origin)?;
			ensure!(Self::transit_nodes().contains(&transit_node), Error::<T>::TransitPointNotFound);

			RouteCosts::<T>::remove_prefix(&transit_node, None);
			for node in Self::transit_nodes() {
				if node == transit_node {
					continue;
				}
				if RouteCosts::<T>::contains_key(&node, &transit_node) {
					RouteCosts::<T>::remove(&node, &transit_node);
				}
			}

			let transit_point_counts = Self::count_for_transit_point().checked_sub(1).ok_or(ArithmeticError::Underflow)?;
			let mut new_transit_nodes = Self::transit_nodes();
			new_transit_nodes.retain(|nodes| *nodes == transit_node);

			CountForTransitPoints::<T>::put(transit_point_counts);
			TransitNodes::<T>::put(new_transit_nodes);

			Self::deposit_event(Event::TransitPointRemoved(transit_node));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_shipment(origin: OriginFor<T>, destination: T::AccountId) -> DispatchResult {

			let transit_node = ensure_signed(origin)?;

			let shipment_uid = Self::shipment_uid().checked_add(1).ok_or(ArithmeticError::Overflow)?;
			
			Self::get_random_route(transit_node.clone(), destination.clone());

			//let route1 = Self::route_vec(transit_node.clone(),destination.clone()).unwrap();

			let shipment = Shipment::<T> {
				creator: transit_node.clone(),
				fees: None, // Todo: Calculate fees based on the route
				owner_index: 1,
				route: Self::get_random_route(transit_node.clone(),destination.clone()),
				destination: destination.clone(),
				uid: shipment_uid.clone(),
				status: ShipmentStatus::InTransit
			};

			ensure!(!UIDToShipment::<T>::contains_key(&shipment_uid), Error::<T>::ShipmentAlreadyExists);
			UIDToShipment::<T>::insert(&shipment_uid, &shipment);

			let key = Self::gen_key();
			UIDToKey::<T>::insert(&shipment_uid, &key);
			ShipmentUID::<T>::put(shipment_uid);

			Self::deposit_event(Event::ShipmentCreated(transit_node));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn update_shipment(origin: OriginFor<T>, shipment_uid: u64, key: [u8; 16]) -> DispatchResult {

			let transit_node = ensure_signed(origin)?;
			let mut shipment = Self::uid_to_shipment(shipment_uid).ok_or(Error::<T>::ShipmentNotFound)?;

			ensure!(UIDToKey::<T>::contains_key(&shipment_uid), Error::<T>::UIDNotFound);
			ensure!(Self::shipment_uid_to_key(&shipment_uid).unwrap() == key, Error::<T>::InvalidKey);
			ensure!(UIDToShipment::<T>::contains_key(&shipment_uid), Error::<T>::ShipmentNotFound);
			ensure!(&transit_node == shipment.route.get(shipment.owner_index as usize).unwrap(), Error::<T>::UnauthorizedCaller);

			UIDToKey::<T>::remove(&shipment_uid);

			match transit_node == shipment.destination {
				true => {
					// Shipment has reached end destination
					shipment.owner_index = 0;
					shipment.status = ShipmentStatus::Delivered;
					UIDToShipment::<T>::insert(&shipment_uid, &shipment);
					Self::deposit_event(Event::ShipmentReceived(transit_node));
				},
				false => {
					// Shipment is still in transit
					shipment.owner_index = shipment.owner_index + 1;
					let new_key = Self::gen_key();
					UIDToKey::<T>::insert(&shipment_uid, &new_key);
					UIDToShipment::<T>::insert(&shipment_uid, &shipment);
					Self::deposit_event(Event::ShipmentUpdated(transit_node));
				}
			}

			Ok(())
		}
	}

	// Helpful functions
	impl<T: Config> Pallet<T> {

		fn gen_key() -> [u8; 16] {
			let payload = (
				T::KeyRandomNess::random(&b"key"[..]).0,
				<frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default(),
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		fn get_and_increment_nonce() -> Vec<u8> {
			let nonce = Nonce::<T>::get();
			Nonce::<T>::put(nonce.wrapping_add(1));
			nonce.encode()
		}

		fn get_random_route(origin: T::AccountId, dest: T::AccountId) -> BoundedVec<T::AccountId,T::MaxSize> {

			let count: u64 = Self::count_for_transit_point();
			let mut route: BoundedVec<_, _>;
			let nodes = TransitNodes::<T>::get();

			if count < 3 {
				//let route_vec1: BoundedVec<_, _> = bounded_vec![origin.clone(),dest.clone()];
				//let route_vec2: BoundedVec<_, _> = bounded_vec![dest.clone(),origin.clone()];

				route = vec![origin.clone(),dest.clone()].try_into().unwrap();

				//RouteVector::<T>::insert(origin.clone(),dest.clone(),vec![origin.clone(),dest.clone()]);
				//RouteVector::<T>::insert(dest.clone(),origin.clone(),vec![dest.clone(),origin.clone()]);
			} else if count >= 3 && count <= 5 {
				let nonce = Self::get_and_increment_nonce();
				let rv1 = T::KeyRandomNess::random(&nonce).encode();
				let mut rn1 = u64::decode(&mut rv1.as_ref()).unwrap();
				let div2: u64 = 2;

				let rv2 = rn1 % div2;

				match rv2 {
					0 => {
						route = vec![origin.clone(),dest.clone()].try_into().unwrap();
					},
					1 => {
						let nonce2 = Self::get_and_increment_nonce();      
						let mut rv3 = T::KeyRandomNess::random(&nonce2).encode();
						rn1 = u64::decode(&mut rv3.as_ref()).unwrap();
						//let count2 = count - 2;

						let mut rv4 = rn1 % count;

						while nodes[rv4 as usize] == origin || nodes[rv4 as usize] == dest {
							let nonce3 = Self::get_and_increment_nonce();    
							rv3 = T::KeyRandomNess::random(&nonce3).encode();  
							rn1 = u64::decode(&mut rv3.as_ref()).unwrap();
							rv4 = rn1 % count;
						}

						route = vec![origin.clone(),nodes[rv4 as usize].clone(),dest.clone()].try_into().unwrap();
 					},
					 _ => {
						 route = vec![].try_into().unwrap();
					 }
				}
			} else  {
				let nonce = Self::get_and_increment_nonce();
				let rv1 = T::KeyRandomNess::random(&nonce).encode();
				let mut rn1 = u64::decode(&mut rv1.as_ref()).unwrap();
				let div3: u64 = 3;

				let rv2 = rn1 % div3;
				match rv2 {
					0 => {
						route = vec![origin.clone(),dest.clone()].try_into().unwrap();
					},
					1 => {
						let nonce2 = Self::get_and_increment_nonce();      
						let mut rv3 = T::KeyRandomNess::random(&nonce2).encode();
						rn1 = u64::decode(&mut rv3.as_ref()).unwrap();

						let mut rv4 = rn1 % count;

						while nodes[rv4 as usize] == origin || nodes[rv4 as usize] == dest {
							let nonce3 = Self::get_and_increment_nonce();    
							rv3 = T::KeyRandomNess::random(&nonce3).encode();  
							rn1 = u64::decode(&mut rv3.as_ref()).unwrap();
							rv4 = rn1 % count;
						}

						route = vec![origin.clone(),nodes[rv4 as usize].clone(),dest.clone()].try_into().unwrap();

					},
					2 => {
						let nonce2 = Self::get_and_increment_nonce();
						let mut rv3 = T::KeyRandomNess::random(&nonce2).encode();
						rn1 = u64::decode(&mut rv3.as_ref()).unwrap();
						

						let mut rv4 = rn1 % count;

						while nodes[rv4 as usize] == origin || nodes[rv4 as usize] == dest {
							let nonce3 = Self::get_and_increment_nonce();    
							rv3 = T::KeyRandomNess::random(&nonce3).encode();
							rn1 = u64::decode(&mut rv3.as_ref()).unwrap();  
							rv4 = rn1 % count;
						}

						let nonce5 = Self::get_and_increment_nonce();    
						rv3 = T::KeyRandomNess::random(&nonce5).encode();
						rn1 = u64::decode(&mut rv3.as_ref()).unwrap();  
						let mut rv5 = rn1 % count;

						while nodes[rv5 as usize] == origin || nodes[rv5 as usize] == dest || rv5 == rv4 {
							let nonce6 = Self::get_and_increment_nonce();    
							rv3 = T::KeyRandomNess::random(&nonce6).encode();
							rn1 = u64::decode(&mut rv3.as_ref()).unwrap();  
							rv5 = rn1 % count;
						}

						route = vec![origin.clone(),nodes[rv4 as usize].clone(),nodes[rv5 as usize].clone(),dest.clone()].try_into().unwrap();

					},
					_ => {
						route = vec![].try_into().unwrap();
					}
				}
			}

			route
		}

		// fn set_fees() {}

		// fn route() {}

		// fn get_transit_nodes() {}

		// fn get_transit_status() {}

		// I think we should be looking to develop an encode decode algorithm. Public key is encoded but only the decoded private key
		// can be used to call the update function.

		// fn get_key(origin: OriginFor<T>, shipment_uid: u64) -> Result<[u8; 16], Error<T>> {
		// 	let transit_node = match ensure_signed(origin) {
		// 		Ok(val) => val,
		// 		Err(_) => return Err(Error::<T>::UnauthorizedCaller)
		// 	};
		// 	ensure!(Self::transit_nodes().contains(&transit_node), Error::<T>::UnauthorizedCaller);
		// 	ensure!(ShipmentUidToKey::<T>::contains_key(&shipment_uid), Error::<T>::UIDNotFound);

		// 	return ShipmentUidToKey::<T>::get(&shipment_uid).ok_or(Error::<T>::KeyNotFound);
		// }
	}
  }
