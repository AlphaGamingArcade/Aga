// The Licensed Work is (c) 2022 Sygma
// SPDX-License-Identifier: LGPL-3.0-only

#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{H160, U256};
use scale_info::TypeInfo;
use scale_info::prelude::vec::Vec;
use codec::{Decode, Encode};

pub type DomainID = u8;
pub type DepositNonce = u64;
pub type ChainID = U256;
pub type VerifyingContractAddress = H160;
pub type DestChainRecipient = Vec<u8>;
pub type DestinationLocation = (Vec<u8>, DomainID);
pub type AssetId = u32;
pub type ResourceId = [u8; 32];
pub const AGA_NATIVE_ASSET_ID: u32 = 0;


#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum TransferType {
	FungibleTransfer,
	NonFungibleTransfer,
	GenericTransfer,
}

pub trait DecimalConverter {
	fn convert_to(amount: u128, decimal: u8) -> u128;
	fn convert_from(amount: u128, decimal: u8) -> u128;
}

/// Simple index type with which we can count sessions.
pub type SessionIndex = u32;
