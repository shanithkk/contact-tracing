#![cfg_attr(not(feature = "std"), no_std)]
use codec::alloc::string::ToString;
use core::convert::TryInto;

use codec::{Encode, Decode};
use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
	sp_runtime::{
		RuntimeDebug,
		DispatchError,
		offchain::{
        self as rt_offchain,
        storage::StorageValueRef,
        storage_lock::{StorageLock, Time},
		},
		traits::{AtLeast32Bit, Bounded, Member, Hash},
	},
	traits::{Get, EnsureOrigin},
	sp_std::{
		prelude::*,
		str,
		vec::Vec,
	},
};
use frame_system::{self as system, ensure_signed, offchain::SendTransactionTypes};
use uuid::Uuid;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const LOCK_TIMEOUT_EXPIRATION: u64 = 3000; // in milli-seconds

use exchangable_id::{ExchangableId,UUID, ExchangableIdProvider};

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Contact<T: Trait, Moment> {
    pub id: ExchangableId<T>,
    pub contact_id: ExchangableId<T>,
    pub timestamp: Moment,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum FlagType {
	Positive,
	Negative,
	Suspicious
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Flag<T: Trait, Moment> {
	pub id: ExchangableId<T>,
	pub flag_type: Option<FlagType>,
	pub timestamp: Moment,
}

pub trait Trait: system::Trait + timestamp::Trait + SendTransactionTypes<Call<Self>> {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type ExchangableIdProvider: ExchangableIdProvider<Self>;
}

decl_storage! {
	trait Store for Module<T: Trait> as ContactTracing {
		pub Contacts get(fn contacts): double_map hasher(blake2_128_concat) ExchangableId<T>, hasher(blake2_128_concat) ExchangableId<T> => Option<Contact<T, T::Moment>>;
		pub Flags get(fn flags): map hasher(blake2_128_concat) ExchangableId<T> => Option<Flag<T, T::Moment>>;
        // pub EventCount get(fn event_count): u128 = 0;
        // // pub AllEvents get(fn event_by_idx): map hasher(blake2_128_concat) ContactEventIndex => Option<ContactEvent<T::Moment>>;
        // // pub EventsOfShipment get(fn events_of_shipment): map hasher(blake2_128_concat) ShipmentId => Vec<ShippingEventIndex>;

        // // // Off-chain Worker notifications
		// // pub OcwNotifications get (fn ocw_notifications): map hasher(identity) T::BlockNumber => Vec<ShippingEventIndex>;
		
		// // Contact uid, id, timestamp

		Something get(fn something): Option<u32>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where 
		AccountId = <T as frame_system::Trait>::AccountId,
		UUID = Vec<u8>,
		ExchangableId = <T as system::Trait>::Hash,
		FlagType = FlagType,
	{
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ContactAdded(ExchangableId),
		ContactFlagged(UUID, FlagType),
		SomethingStored(u32, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NotOwner,
		InvalidUUID,
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		/// 
		#[weight = 0]
		pub fn add_contact(origin, id: UUID, contact_id: UUID) {
			let sender = ensure_signed(origin)?;
			ensure!(
				T::ExchangableIdProvider::check_uuid_exists(&id),
				Error::<T>::InvalidUUID,
			);
			ensure!(
				T::ExchangableIdProvider::check_uuid_exists(&contact_id),
				Error::<T>::InvalidUUID,
        	);
			let id = T::Hashing::hash_of(&Uuid::parse_str(str::from_utf8(&id).unwrap()).unwrap().as_bytes());
			let contact_id = T::Hashing::hash_of(&Uuid::parse_str(str::from_utf8(&contact_id).unwrap()).unwrap().as_bytes());
			let contact = Self::new_contact(id, contact_id);

			// Self::deposit_event(RawEvent::ContactAdded(contact.id));
		}

		#[weight = 0]
		pub fn check_uuid(origin, uuid: UUID) {
			ensure!(
				T::ExchangableIdProvider::check_uuid_exists(&uuid),
				Error::<T>::InvalidUUID,
        	);
		}

		#[weight = 0]
		pub fn add_flag(origin, id: UUID, flag_type: FlagType) {
			let sender = ensure_signed(origin)?;
			let id = T::Hashing::hash_of(&Uuid::parse_str(str::from_utf8(&id).unwrap()).unwrap().as_bytes());

			// Needed fix
			// ensure!(
			// 	T::ExchangableIdProvider::check_exchangable_id_owner(sender, id),
			// 	Error::<T>::NotOwner,
			// );
			
			let flag_type_event = flag_type.clone();
			Self::new_flag(id, flag_type);
			let uuid = T::ExchangableIdProvider::get_uuid(id);
			Self::deposit_event(RawEvent::ContactFlagged(uuid.unwrap(), flag_type_event));
		}

		#[weight = 0]
		pub fn check_flag(origin, id: UUID) {

		}

		// #[weight = 10_000 + T::DbWeight::get().writes(1)]
		// pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
		// 	// Check that the extrinsic was signed and get the signer.
		// 	// This function will return an error if the extrinsic is not signed.
		// 	// https://substrate.dev/docs/en/knowledgebase/runtime/origin
		// 	let who = ensure_signed(origin)?;

		// 	// Update storage.
		// 	Something::put(something);

		// 	// Emit an event.
		// 	Self::deposit_event(RawEvent::SomethingStored(something, who));
		// 	// Return a successful DispatchResult
		// 	Ok(())
		// }

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}

		fn offchain_worker(block_number: T::BlockNumber) {

			//debug::info!("offchain worker");
			let block_hash = <system::Module<T>>::block_hash(block_number);
			//debug::warn!("Current block is: {:?} (parent: {:?})", block_number, block_hash);

            // Acquiring the lock
            let mut lock = StorageLock::<Time>::with_deadline(
                b"contact_tracing_ocw::lock",
                rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION)
            );

            match lock.try_lock() {
                Ok(_guard) => { Self::process_ocw_notifications(block_number); }
                Err(_err) => { debug::info!("[contact_tracing_ocw] lock is already acquired"); }
            };
		}
		
	}
}

impl<T: Trait> Module<T> {

	fn new_contact(id: ExchangableId<T>, contact_id: ExchangableId<T>) -> Contact<T, T::Moment>{
		let contact = Contact::<T, T::Moment> {
			id: id,
			contact_id: contact_id,
			timestamp: <timestamp::Module<T>>::now(),
		};
		Contacts::insert(id, contact_id, &contact);
		contact
	}

	fn new_flag(id: ExchangableId<T>, flag_type: FlagType) {
		let flag = Flag::<T, T::Moment> {
			id: id,
			flag_type: Some(flag_type),
			timestamp: <timestamp::Module<T>>::now(),
		};
		Flags::insert(id,&flag);
	}

	pub fn check_exchangable_id_exists(props: &ExchangableId<T>) -> Result<(), Error<T>> {
        // ensure!(
        //     props.len() <= SHIPMENT_MAX_PRODUCTS,
        //     Error::<T>::ShipmentHasTooManyProducts,
        // );
        Ok(())
    }
	
	fn process_ocw_notifications(block_number: T::BlockNumber) {
		let last_processed_block_ref = StorageValueRef::persistent(b"contact_tracing_ocw::last_proccessed_block");
		let mut last_processed_block: u32 = match last_processed_block_ref.get::<T::BlockNumber>() {
			Some(Some(last_proccessed_block)) if last_proccessed_block >= block_number => {
				debug::info!(
					"[contact_tracing_ocw] Skipping: Block {:?} has already been processed.",
					block_number
				);
				return;
			}
			Some(Some(last_proccessed_block)) => {
				last_proccessed_block.try_into().ok().unwrap() as u32
			}
			None => 0, //TODO: define a OCW_MAX_BACKTRACK_PERIOD param
			_ => 	{
				debug::error!("[contact_tracing_ocw] Error reading contact_tracing_ocw::last_proccessed_block.");
				return;
			}
		};
		
		let start_block = last_processed_block + 1;
		let end_block = block_number.try_into().ok().unwrap() as u32;
		for current_block in start_block..end_block {
			debug::debug!(
				"[contact_tracing_ocw] Processing notifications for block {}",
				current_block
			);
			// let ev_indices = Self::ocw_notifications::<T::BlockNumber>(current_block.into());

			// let listener_results: Result<Vec<_>, _> = ev_indices
			// 	.iter()
			// 	.map(|idx| match Self::event_by_idx(idx) {
			// 		Some(ev) => Self::notify_listener(&ev),
			// 		None => Ok(()),
			// 	})
			// 	.collect();

			// if let Err(err) = listener_results {
			// 	debug::warn!("[contact_tracing_ocw] notify_listener error: {}", err);
			// 	break;
			// }
			last_processed_block = current_block;
		}
	}
}

// #[derive(Default)]
// pub struct ContactBuilder<T: Trait, Moment>
// where
// {
//     id: Option<ExchangableId<T>>,
//     contact_id: Option<ExchangableId<T>>,
//     timestamp: Moment,
// }

// impl<T: Trait, Moment> ContactBuilder<T, Moment>
// where
// {
// 	pub fn add_id(mut self, id: ExchangableId<T>)-> Self {
//         self.id = id;
//         self
// 	}
	
// 	pub fn add_contact_id(mut self, contact_id: ExchangableId<T>)-> Self {
//         self.contact_id = contact_id;
//         self
//     }

// 	pub fn add_timestamp(mut self, timestamp: Moment) -> Self {
//         self.timestamp = timestamp;
//         self
//     }

// 	pub fn build(self) -> Contact<T, Moment> {
//         Contact::<T, Moment> {
//             id: self.id,
//             contact_id: self.contact_id,
//             timestamp: self.timestamp,
//         }
// 	}
	

// }
