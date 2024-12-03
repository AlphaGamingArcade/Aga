// The Licensed Work is (c) 2022 Sygma
// SPDX-License-Identifier: LGPL-3.0-only

#![cfg_attr(not(feature = "std"), no_std)]

pub use self::pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use aga_primitives::{
	ChainID, DomainID, DepositNonce
};

#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::fungible::{NativeOrWithId, Inspect, Mutate};
	use frame_support::traits::tokens::Preservation;
	use scale_info::prelude::vec::Vec;
	use primitive_types::U256;
	use frame_support::PalletId;
	use sp_io::hashing::keccak_256;
	use frame_support::sp_runtime::traits::AccountIdConversion;
	use frame_support::sp_runtime::{SaturatedConversion, Saturating};

	#[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo, RuntimeDebug)]
	pub struct Proposal {
		pub origin_domain_id: DomainID,
		pub deposit_nonce: DepositNonce,
		pub resource_id: [u8; 32],
		pub data: Vec<u8>,
	}

	// Allows easy access our Pallet's `Balance` type. Comes from `Fungible` interface.
    pub type BalanceOf<T> =
        <<T as Config>::NativeBalances as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	// Allows Easy access to accounts
	// let admin = T::Lookup::lookup(admin)?;
	// pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	// This includes the native and assets type
	pub type NativeAndAssetId = NativeOrWithId<u32>;

	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	///
	/// All our types and constants a pallet depends on must be declared here.
	/// These types are defined generically and made concrete when the pallet is declared in the
	/// `runtime/src/lib.rs` file of your chain.
	#[pallet::config]
	pub trait Config: frame_system::Config + aga_access_segregator::Config  {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// the actual balance
		type NativeBalances: Inspect<Self::AccountId> + Mutate<Self::AccountId>;
		/// Type of asset class, sourced from [`Config::Assets`], utilized to offer liquidity to a
		/// pool.
		type AssetKind: Parameter + MaxEncodedLen;
		/// Transfer reserve account holder
		type TransferReserveAccount: Get<Self::AccountId>;
		/// Transfer reserve account holder
		type FeeReserveAccount: Get<Self::AccountId>;
		/// Config ID for the current pallet instance
		type PalletId: Get<PalletId>;
		/// ResourceId type for 32-byte identifiers.
		type ResourceId: Get<[u8; 32]>;
		/// Current pallet index defined in runtime
		type PalletIndex: Get<u8>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	/// Mark supported dest domainID
	#[pallet::storage]
	#[pallet::getter(fn dest_domain_ids)]
	pub type DestDomainIds<T: Config> = StorageMap<_, Twox64Concat, DomainID, bool, ValueQuery>;

	/// Mark the pairs for supported dest domainID with its corresponding chainID
	/// The chainID is not directly used in pallet, this map is designed more about rechecking the
	/// domainID
	#[pallet::storage]
	#[pallet::getter(fn dest_chain_ids)]
	pub type DestChainIds<T: Config> = StorageMap<_, Twox64Concat, DomainID, ChainID>;

	/// Mapping fungible asset id to corresponding fee amount
	#[pallet::storage]
	#[pallet::getter(fn asset_fees)]
	pub type AssetFees<T: Config> = StorageMap<_, Twox64Concat, DomainID, u128>;

	/// Deposit counter of dest domain
	#[pallet::storage]
	#[pallet::getter(fn deposit_counts)]
	pub type DepositCounts<T> = StorageMap<_, Twox64Concat, DomainID, DepositNonce, ValueQuery>;

	/// Bridge Pause indicator
	/// Bridge is unpaused initially, until pause
	/// After granted access setup, bridge should be paused until ready to unpause
	#[pallet::storage]
	#[pallet::getter(fn is_paused)]
	pub type IsPaused<T> = StorageMap<_, Twox64Concat, DomainID, bool, ValueQuery>;


	/// Mark whether a deposit nonce was used. Used to mark execution status of a proposal.
	#[pallet::storage]
	#[pallet::getter(fn used_nonces)]
	pub type UsedNonces<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		DomainID,
		Twox64Concat,
		DepositNonce,
		DepositNonce,
		ValueQuery,
	>;
	

	#[allow(dead_code)]
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deposit
		Deposit { 
			dest_domain_id: DomainID,
			resource_id: [u8; 32],
			sender: T::AccountId,
			deposit_nonce: DepositNonce,
			deposit_data: Vec<u8>,
		},
		/// When proposal was executed successfully
		ProposalExecution {
			origin_domain_id: DomainID,
			deposit_nonce: DepositNonce,
			data_hash: [u8; 32],
		},
		/// When registering a new dest domainID with its corresponding chainID
		RegisterDestDomain { 
			sender: T::AccountId, 
			domain_id: DomainID, 
			chain_id: ChainID 
		},
		/// When unregistering dest domainID with its corresponding chainID
		UnregisterDestDomain { 
			sender: T::AccountId, 
			domain_id: DomainID, 
			chain_id: ChainID 
		},
		/// A user has successfully set a new value.
		FeeSet {
			/// The new value set.
			domain_id: DomainID,
			/// Amount
			amount: u128
		},
		/// When bridge fee is collected
		FeeCollected {
			fee_payer: T::AccountId,
			dest_domain_id: DomainID,
			fee_amount: u128,
		},
		/// When proposal was faild to execute
		FailedHandlerExecution {
			error: Vec<u8>,
			origin_domain_id: DomainID,
			deposit_nonce: DepositNonce,
		},
		/// When all bridges are paused
		AllBridgePaused { sender: T::AccountId },
		/// When all bridges are unpaused
		AllBridgeUnpaused { sender: T::AccountId },
		/// When bridge is paused
		/// args: [dest_domain_id]
		BridgePaused { dest_domain_id: DomainID },
		/// When bridge is unpaused
		/// args: [dest_domain_id]
		BridgeUnpaused { dest_domain_id: DomainID },
		/// TEST 
		TestAddress { address: Vec<u8> }
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account has not gained access permission
		AccessDenied,
		DestDomainNotSupported,
		DestChainIDNotMatch,
		MissingFeeConfig,
		InsufficientBalance,
		DepositNonceOverflow,
		EmptyProposalList,
		ProposalAlreadyComplete,
		InvalidDepositData,
		Overflow,
		Underflow,
		InvalidDecimalConversion,
		DecodingAccountIdFailed,
		InvalidAccountId,
		/// Bridge is paused
		BridgePaused,
		/// Bridge is unpaused
		BridgeUnpaused,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Pause bridge, this would lead to bridge transfer failure before it being unpaused.
		#[pallet::call_index(0)]
		#[pallet::weight(< T as Config >::WeightInfo::pause_bridge())]
		pub fn pause_bridge(origin: OriginFor<T>, dest_domain_id: DomainID) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"pause_bridge".to_vec(),
					origin
				),
				Error::<T>::AccessDenied
			);
			ensure!(DestDomainIds::<T>::get(dest_domain_id), Error::<T>::DestDomainNotSupported);

			// Mark as paused
			IsPaused::<T>::insert(dest_domain_id, true);

			// Emit BridgePause event
			Self::deposit_event(Event::BridgePaused { dest_domain_id });
			Ok(())
		}

		/// Unpause bridge.
		#[pallet::call_index(1)]
		#[pallet::weight(< T as Config >::WeightInfo::unpause_bridge())]
		pub fn unpause_bridge(origin: OriginFor<T>, dest_domain_id: DomainID) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"unpause_bridge".to_vec(),
					origin
				),
				Error::<T>::AccessDenied
			);
			ensure!(DestDomainIds::<T>::get(dest_domain_id), Error::<T>::DestDomainNotSupported);

			// make sure the current status is paused
			ensure!(IsPaused::<T>::get(dest_domain_id), Error::<T>::BridgeUnpaused);

			// Mark as unpaused
			IsPaused::<T>::insert(dest_domain_id, false);

			// Emit BridgeUnpause event
			Self::deposit_event(Event::BridgeUnpaused { dest_domain_id });
			Ok(())
		}


		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_domain())]
		pub fn register_domain(
			origin: OriginFor<T>,
			dest_domain_id: DomainID,
			dest_chain_id: ChainID
		) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"register_domain".to_vec(),
					origin.clone()
				),
				Error::<T>::AccessDenied
			);

			DestDomainIds::<T>::insert(dest_domain_id, true);
			DestChainIds::<T>::insert(dest_domain_id, dest_chain_id);

			// Emit register dest domain event
			let sender = ensure_signed(origin)?;

			Self::deposit_event(Event::RegisterDestDomain {
				sender,
				domain_id: dest_domain_id,
				chain_id: dest_chain_id,
			});

			Ok(())
		}

		/// Mark the give dest domainID with chainID to be disabled
		#[pallet::call_index(3)]
		#[pallet::weight(< T as Config >::WeightInfo::unregister_domain())]
		pub fn unregister_domain(
			origin: OriginFor<T>,
			dest_domain_id: DomainID,
			dest_chain_id: ChainID,
		) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"unregister_domain".to_vec(),
					origin.clone()
				),
				Error::<T>::AccessDenied
			);
			ensure!(
				DestDomainIds::<T>::get(dest_domain_id)
					&& DestChainIds::<T>::get(dest_domain_id).is_some(),
				Error::<T>::DestDomainNotSupported
			);

			let co_chain_id = DestChainIds::<T>::get(dest_domain_id).unwrap();
			ensure!(co_chain_id == dest_chain_id, Error::<T>::DestChainIDNotMatch);

			DestDomainIds::<T>::remove(dest_domain_id);
			DestChainIds::<T>::remove(dest_domain_id);

			// Emit register dest domain event
			let sender = ensure_signed(origin)?;

			Self::deposit_event(Event::UnregisterDestDomain {
				sender,
				domain_id: dest_domain_id,
				chain_id: dest_chain_id,
			});
			
			Ok(())
		}

		/// Mark the give dest domainID with chainID to be disabled
		#[pallet::call_index(4)]
		#[pallet::weight(< T as Config >::WeightInfo::deposit())]
		pub fn deposit(
			origin: OriginFor<T>,
			dest_domain_id: DomainID,
			deposit_data: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(
				DestDomainIds::<T>::get(dest_domain_id), 
				Error::<T>::DestDomainNotSupported
			);
			
			// Get the sender's current balance
			let sender_balance = T::NativeBalances::balance(&sender);

			let (amount, recipient) = Self::extract_deposit_data(&deposit_data)?;
			
			ensure!(!IsPaused::<T>::get(dest_domain_id), Error::<T>::BridgePaused);

			// Let total amount to send in balance type
			let amount_in_balance = amount.saturated_into::<BalanceOf<T>>();

			// Ensure the sender has enough balance to cover both the transfer and the fee
			ensure!(
				sender_balance >= amount_in_balance, 
				Error::<T>::InsufficientBalance
			);

			// Ensure fee is set
			let fee = AssetFees::<T>::get(dest_domain_id)
				.ok_or(Error::<T>::MissingFeeConfig)?;

			let fee_in_balance = fee.saturated_into::<BalanceOf<T>>();
			let fee_reserve_account = T::FeeReserveAccount::get();
			T::NativeBalances::transfer(
				&sender,
				&fee_reserve_account,
				fee_in_balance,
				Preservation::Expendable // Allow death
			)?;

			Self::deposit_event(Event::FeeCollected {
				fee_payer: sender.clone(),
				dest_domain_id,
				fee_amount: fee,
			});

			// net_amount_in_balance is amount after fees
			let net_amount_in_balance = amount_in_balance.saturating_sub(fee_in_balance);
			let transfer_reserve_account = T::TransferReserveAccount::get();
			T::NativeBalances::transfer(
				&sender,
				&transfer_reserve_account,
				net_amount_in_balance,
				Preservation::Expendable // Allow death
			)?;

			// Bump deposit nonce
			let deposit_nonce = DepositCounts::<T>::get(dest_domain_id);
			DepositCounts::<T>::insert(
				dest_domain_id,
				deposit_nonce.checked_add(1).ok_or(Error::<T>::DepositNonceOverflow)?,
			);

			// net_amount is the amount after all fees were deducted
			let net_amount = amount
				.checked_sub(fee)
				.ok_or(Error::<T>::Underflow)?;
			
			Self::deposit_event(Event::Deposit {
				dest_domain_id,
				resource_id: T::ResourceId::get(),
				sender,
				deposit_nonce,
				deposit_data: Self::create_deposit_data(net_amount, recipient),
			});
			
			Ok(())
		}

		/// Mark the give dest domainID with chainID to be disabled
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config >::WeightInfo::execute_proposals(proposals.len() as u32))]
		pub fn execute_proposals(
			origin: OriginFor<T>,
			proposals: Vec<Proposal>,
		) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"execute_proposals".to_vec(),
					origin.clone()
				),
				Error::<T>::AccessDenied
			);
			// Ensure propsals is not empty
			ensure!(!proposals.is_empty(), Error::<T>::EmptyProposalList);
			// Execute proposals one by one.
			// Note if one proposal failed to execute, we emit `FailedHandlerExecution` rather
			// than revert whole transaction
			for proposal in proposals.iter() {
				Self::execute_proposal_internal(proposal).map_or_else(
					|e| {
						let err_msg: &'static str = e.into();
						// Any error during proposal list execution will emit FailedHandlerExecution
						Self::deposit_event(Event::FailedHandlerExecution {
							error: err_msg.as_bytes().to_vec(),
							origin_domain_id: proposal.origin_domain_id,
							deposit_nonce: proposal.deposit_nonce,
						});
					},
					|_| {
						// Update proposal status
						Self::set_proposal_executed(
							proposal.deposit_nonce,
							proposal.origin_domain_id,
						);

						// Emit ProposalExecution
						Self::deposit_event(Event::ProposalExecution {
							origin_domain_id: proposal.origin_domain_id,
							deposit_nonce: proposal.deposit_nonce,
							data_hash: keccak_256(
								&[
									proposal.data.clone(),
									T::PalletId::get().into_account_truncating(),
								]
								.concat(),
							),
						});
					},
				);
			}

			Ok(())
		}

		/// Mark the give dest domainID with chainID to be disabled
		#[pallet::call_index(6)]
		#[pallet::weight(< T as Config >::WeightInfo::set_fee())]
		pub fn set_fee(
			origin: OriginFor<T>,
			domain_id: DomainID,
			amount: u128,
		) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"set_fee".to_vec(),
					origin
				),
				Error::<T>::AccessDenied
			);
			
			// Ensure that domain is registered
			ensure!(DestDomainIds::<T>::get(domain_id), Error::<T>::DestDomainNotSupported);

			// Update asset fee
			AssetFees::<T>::insert(domain_id, amount);

			// Emit FeeSet event
			Self::deposit_event(Event::FeeSet { domain_id, amount });

			Ok(())
		}

		/// Pause all registered bridges
		#[pallet::call_index(7)]
		#[pallet::weight(< T as Config >::WeightInfo::pause_all_bridges())]
		pub fn pause_all_bridges(origin: OriginFor<T>) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"pause_all_bridges".to_vec(),
					origin.clone()
				),
				Error::<T>::AccessDenied
			);

			// Pause all bridges
			Self::pause_all_domains();

			// Emit AllBridgePaused
			let sender = ensure_signed(origin)?;
			Self::deposit_event(Event::AllBridgePaused { sender });

			Ok(())
		}

		/// Unpause all registered bridges
		#[pallet::call_index(8)]
		#[pallet::weight(< T as Config >::WeightInfo::unpause_all_bridges())]
		pub fn unpause_all_bridges(origin: OriginFor<T>) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"unpause_all_bridges".to_vec(),
					origin.clone()
				),
				Error::<T>::AccessDenied
			);

			// Unpause all bridges
			Self::unpause_all_domains();

			// Emit AllBridgeUnpaused
			let sender = ensure_signed(origin)?;
			Self::deposit_event(Event::AllBridgeUnpaused { sender });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn create_deposit_data(amount: u128, recipient: Vec<u8>) -> Vec<u8> {
			[
				&Self::hex_zero_padding_32(amount),
				&Self::hex_zero_padding_32(recipient.len() as u128), // ETHEREUM Address length
				recipient.as_slice(),
			]
			.concat()
			.to_vec()
		}

		fn hex_zero_padding_32(i: u128) -> [u8; 32] {
			let mut result = [0u8; 32];
			U256::from(i).to_big_endian(&mut result);
			result
		}

		/// Return true if deposit nonce has been used
		pub fn is_proposal_executed(nonce: DepositNonce, domain_id: DomainID) -> bool {
			(UsedNonces::<T>::get(domain_id, nonce / 64) & (1 << (nonce % 64))) != 0
		}

		/// Set bit mask for specific nonce as used
		fn set_proposal_executed(nonce: DepositNonce, domain_id: DomainID) {
			let mut current_nonces = UsedNonces::<T>::get(domain_id, nonce / 64);
			current_nonces |= 1 << (nonce % 64);
			UsedNonces::<T>::insert(domain_id, nonce / 64, current_nonces);
		}
		
		/// Execute a single proposal
		fn execute_proposal_internal(proposal: &Proposal) -> DispatchResult {
			// Check if domain is supported
			ensure!(
				DestDomainIds::<T>::get(proposal.origin_domain_id),
				Error::<T>::DestDomainNotSupported
			);

			// Check if proposal has executed
			ensure!(
				!Self::is_proposal_executed(proposal.deposit_nonce, proposal.origin_domain_id),
				Error::<T>::ProposalAlreadyComplete
			);

			let (amount, recipient) = Self::extract_deposit_data(&proposal.data)?;
			
			// Let total amount to send in balance type
			let amount_balance = amount.saturated_into::<BalanceOf<T>>();

			let token_reserved_account = T::TransferReserveAccount::get();

			// Decode recipient data into AccountId
			let recipient_account_id = T::AccountId::decode(&mut &recipient[..])
				.map_err(|_| Error::<T>::InvalidDepositData)?;

			T::NativeBalances::transfer(
				&token_reserved_account,
				&recipient_account_id,
				amount_balance,
				Preservation::Expendable // Allow death
			)?;

			Ok(())
		}

		fn extract_deposit_data(data: &[u8]) -> Result<(u128, Vec<u8>), DispatchError> {
			if data.len() < 64 {
				return Err(Error::<T>::InvalidDepositData.into());
			}
		
			// Extract the amount as u128
			let amount: u128 = U256::from_big_endian(&data[0..32])
				.try_into()
				.map_err(|_| Error::<T>::InvalidDepositData)?;
		
			// Extract recipient data length
			let recipient_len: usize = U256::from_big_endian(&data[32..64])
				.try_into()
				.map_err(|_| Error::<T>::InvalidDepositData)?;

			if (data.len() - 64) < recipient_len {
				return Err(Error::<T>::InvalidDepositData.into());
			}

			let recipient = data[64..(64 + recipient_len)].to_vec();
			
			// Return the amount and recipient
			Ok((amount, recipient))
		}

		/// unpause all registered domains in the storage
		fn unpause_all_domains() {
			DestDomainIds::<T>::iter_keys().for_each(|d| IsPaused::<T>::insert(d, false));
			IsPaused::<T>::iter_keys().for_each(|d| IsPaused::<T>::insert(d, false));
		}

		/// pause all registered domains in the storage
		fn pause_all_domains() {
			DestDomainIds::<T>::iter_keys().for_each(|d| IsPaused::<T>::insert(d, true));
			IsPaused::<T>::iter_keys().for_each(|d| IsPaused::<T>::insert(d, true));
		}
	}
}

sp_api::decl_runtime_apis! {
	/// This runtime api is for checking if the proposal is executed already
	pub trait AgaBridgeApi {
		fn is_proposal_executed(nonce: DepositNonce, domain_id: DomainID) -> bool;
	}
}
