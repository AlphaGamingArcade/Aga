// The Licensed Work is (c) 2022 Sygma
// SPDX-License-Identifier: LGPL-3.0-only

#![cfg_attr(not(feature = "std"), no_std)]

pub use self::pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::fungible::{NativeOrWithId, Inspect, Mutate};
	use aga_traits::{
		ChainID, DomainID, TransferType, DepositNonce
	};
	use frame_support::traits::tokens::Preservation;
	use scale_info::prelude::vec::Vec;
	use primitive_types::U256;
	use frame_support::PalletId;
	use sp_io::hashing::keccak_256;
	use frame_support::sp_runtime::traits::AccountIdConversion;

	#[derive(PartialEq, Eq, Clone, Encode, Decode, TypeInfo, RuntimeDebug)]
	pub struct Proposal {
		pub origin_domain_id: DomainID,
		pub deposit_nonce: DepositNonce,
		pub data: Vec<u8>,
	}

	// Allows easy access our Pallet's `Balance` type. Comes from `Fungible` interface.
    pub type BalanceOf<T> =
        <<T as Config>::NativeBalances as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
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
	pub type AssetFees<T: Config> = StorageMap<_, Twox64Concat, DomainID, BalanceOf<T>>;

	/// Deposit counter of dest domain
	#[pallet::storage]
	#[pallet::getter(fn deposit_counts)]
	pub type DepositCounts<T> = StorageMap<_, Twox64Concat, DomainID, DepositNonce, ValueQuery>;

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
			sender: T::AccountId,
			deposit_nonce: DepositNonce, 
			transfer_type: TransferType,
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
		/// Transferred
		Transferred { 
			from: T::AccountId,
			to: T::AccountId,
			amount: BalanceOf<T>, 
		},
		/// A user has successfully set a new value.
		FeeSet {
			/// The new value set.
			domain_id: DomainID,
			/// Amount
			amount: BalanceOf<T>
		},
		/// When bridge fee is collected
		FeeCollected {
			fee_payer: T::AccountId,
			dest_domain_id: DomainID,
			fee_amount: BalanceOf<T>,
		},
		/// When proposal was faild to execute
		FailedHandlerExecution {
			error: Vec<u8>,
			origin_domain_id: DomainID,
			deposit_nonce: DepositNonce,
		},
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
		InvalidDepositData
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_domain())]
		pub fn register_domain(
			origin: OriginFor<T>,
			dest_domain_id: DomainID,
			dest_chain_id: ChainID,
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
		#[pallet::call_index(1)]
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
		#[pallet::call_index(2)]
		#[pallet::weight(< T as Config >::WeightInfo::deposit())]
		pub fn deposit(
			origin: OriginFor<T>,
			dest_domain_id: DomainID,
			dest_chain_recipient: Vec<u8>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let transfer_type = TransferType::FungibleTransfer; // Default to fungible for now
			let amount_u128: u128 = amount.try_into().unwrap_or_else(|_| {
				// Handle the conversion error, in case the BalanceOf<T> type is not directly convertible
				// You can return an error or take some default action if required
				panic!("Failed to convert BalanceOf<T> to u128");
			});

			// Ensure that domain is registered
			ensure!(DestDomainIds::<T>::get(dest_domain_id), Error::<T>::DestDomainNotSupported);
			// Ensure fee is set
			let fee = AssetFees::<T>::get(dest_domain_id).ok_or(Error::<T>::MissingFeeConfig)?;
			
			// Get the sender's current balance
			let sender_balance = T::NativeBalances::balance(&sender);

			// Calculate the total amount required (fee + transfer amount)
			let total_amount_needed = fee + amount;

			// Ensure the sender has enough balance to cover both the transfer and the fee
			ensure!(sender_balance >= total_amount_needed, Error::<T>::InsufficientBalance);
			let transfer_reserve_account = T::TransferReserveAccount::get();
			T::NativeBalances::transfer(
				&sender,
				&transfer_reserve_account,
				amount,
				Preservation::Expendable // Allow death
			)?;

			let fee_reserve_account = T::FeeReserveAccount::get();
			T::NativeBalances::transfer(
				&sender,
				&fee_reserve_account,
				fee,
				Preservation::Expendable // Allow death
			)?;

			// Bump deposit nonce
			let deposit_nonce = DepositCounts::<T>::get(dest_domain_id);
			DepositCounts::<T>::insert(
				dest_domain_id,
				deposit_nonce.checked_add(1).ok_or(Error::<T>::DepositNonceOverflow)?,
			);

			Self::deposit_event(Event::FeeCollected {
				fee_payer: sender.clone(),
				dest_domain_id,
				fee_amount: fee,
			});
			
			Self::deposit_event(Event::Deposit {
				dest_domain_id, 
				sender,
				deposit_nonce, 
				transfer_type,
				deposit_data: Self::create_deposit_data(amount_u128, dest_chain_recipient),
			});
			
			Ok(())
		}

		/// Mark the give dest domainID with chainID to be disabled
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config >::WeightInfo::execute_proposal(proposals.len() as u32))]
		pub fn execute_proposal(
			origin: OriginFor<T>,
			proposals: Vec<Proposal>,
		) -> DispatchResult {
			ensure!(
				<aga_access_segregator::pallet::Pallet<T>>::has_access(
					<T as Config>::PalletIndex::get(),
					b"execute_proposal".to_vec(),
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
		#[pallet::call_index(4)]
		#[pallet::weight(< T as Config >::WeightInfo::set_fee())]
		pub fn set_fee(
			origin: OriginFor<T>,
			domain_id: DomainID,
			amount: BalanceOf<T>,
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
	}

	impl<T: Config> Pallet<T> {
		pub fn create_deposit_data(amount: u128, recipient: Vec<u8>) -> Vec<u8> {
			[
				&Self::hex_zero_padding_32(amount),
				&Self::hex_zero_padding_32(recipient.len() as u128),
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

			let token_reserved_account = T::TransferReserveAccount::get();

			T::NativeBalances::transfer(
				&token_reserved_account,
				&recipient,
				amount,
				Preservation::Expendable // Allow death
			)?;

			Ok(())
		}

		fn extract_deposit_data(data: &[u8]) -> Result<(BalanceOf<T>, T::AccountId), DispatchError> {
			if data.len() < 64 {
				return Err(Error::<T>::InvalidDepositData.into());
			}
		
			// Extract the amount as u128
			let amount_u128: u128 = U256::from_big_endian(&data[0..32])
				.try_into()
				.map_err(|_| Error::<T>::InvalidDepositData)?;
		
			// Convert u128 to BalanceOf<T>
			let amount: BalanceOf<T> = amount_u128
				.try_into()
				.map_err(|_| Error::<T>::InvalidDepositData)?;
		
			// Extract recipient data length
			let recipient_len: usize = U256::from_big_endian(&data[32..64])
				.try_into()
				.map_err(|_| Error::<T>::InvalidDepositData)?;
		
			// Ensure recipient length matches the remaining data
			if (data.len() - 64) != recipient_len {
				return Err(Error::<T>::InvalidDepositData.into());
			}
		
			// Decode recipient data into AccountId
			let recipient = T::AccountId::decode(&mut &data[64..data.len()])
			.map_err(|_| Error::<T>::InvalidDepositData)?;
		
			// Return the amount and recipient
			Ok((amount, recipient))
		}
	}
}